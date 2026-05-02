import { Terminal } from "./terminal";
import { TextBuffer } from "./buffer";
import { Renderer } from "./renderer";
import { PluginManager } from "./plugin/manager";
import type { EditorState, Mode } from "./types";

export class Editor {
  private terminal = new Terminal();
  private buffer = new TextBuffer();
  private renderer = new Renderer(this.terminal);
  private pluginManager = new PluginManager(
    () => this.buffer,
    () => this.state
  );
  private state: EditorState = {
    mode: "normal",
    cursor: { line: 1, col: 0 },
    filePath: null,
    dirty: false,
  };
  private running = false;
  private commandBuffer = "";
  private cleaned = false;

  async run(filePath?: string): Promise<void> {
    if (filePath) {
      await this.buffer.loadFile(filePath);
      this.state.filePath = filePath;
      this.renderer.resetScroll();
    }

    await this.pluginManager.loadAll();

    this.terminal.enableRawMode();
    this.terminal.hideCursor();
    this.running = true;

    await this.pluginManager.emit("editor:ready", undefined);

    process.on("exit", () => this.cleanup());
    process.on("SIGINT", () => { this.cleanup(); process.exit(0); });

    try {
      while (this.running) {
        this.state.dirty = this.buffer.dirty;
        this.renderer.render(this.buffer, this.state);
        this.terminal.showCursor();

        const key = await this.terminal.readKey();
        await this.handleKey(key);
      }
    } finally {
      this.cleanup();
    }
  }

  private async handleKey(key: string): Promise<void> {
    switch (this.state.mode) {
      case "normal": await this.handleNormal(key); break;
      case "insert": await this.handleInsert(key); break;
      case "command": await this.handleCommand(key); break;
    }
  }

  private async handleNormal(key: string): Promise<void> {
    await this.pluginManager.emit("key:normal", { key });
    const handled = await this.pluginManager.keymaps.handle("normal", key);
    if (handled) return;

    const { cursor } = this.state;
    const lineCount = this.buffer.lineCount;

    switch (key) {
      case "h": cursor.col = Math.max(0, cursor.col - 1); break;
      case "l": {
        const lineLen = this.buffer.getLine(cursor.line).length;
        cursor.col = Math.min(lineLen > 0 ? lineLen - 1 : 0, cursor.col + 1);
        break;
      }
      case "j": {
        cursor.line = Math.min(lineCount, cursor.line + 1);
        const len = this.buffer.getLine(cursor.line).length;
        cursor.col = Math.min(cursor.col, len > 0 ? len - 1 : 0);
        break;
      }
      case "k": {
        cursor.line = Math.max(1, cursor.line - 1);
        const len = this.buffer.getLine(cursor.line).length;
        cursor.col = Math.min(cursor.col, len > 0 ? len - 1 : 0);
        break;
      }
      case "i": {
        const prevMode = this.state.mode;
        this.state.mode = "insert";
        await this.pluginManager.emit("mode:change", { from: prevMode, to: "insert" });
        break;
      }
      case ":": {
        const prevMode = this.state.mode;
        this.state.mode = "command";
        this.terminal.moveCursor(this.terminal.rows, 1);
        process.stdout.write(":");
        this.commandBuffer = "";
        await this.pluginManager.emit("mode:change", { from: prevMode, to: "command" });
        break;
      }
    }
  }

  private async handleInsert(key: string): Promise<void> {
    await this.pluginManager.emit("key:insert", { key });
    const handled = await this.pluginManager.keymaps.handle("insert", key);
    if (handled) return;

    const { cursor } = this.state;

    if (key === "\x1b") {
      const prevMode = this.state.mode;
      this.state.mode = "normal";
      cursor.col = Math.max(0, cursor.col - 1);
      await this.pluginManager.emit("mode:change", { from: prevMode, to: "normal" });
      return;
    }

    if (key === "\x7f" || key === "\b") {
      if (cursor.col > 0) {
        this.buffer.delete(cursor.line, cursor.col - 1);
        cursor.col--;
        await this.pluginManager.emit("buffer:change", { buffer: this.buffer });
      } else if (cursor.line > 1) {
        const newCol = this.buffer.mergeWithPrevLine(cursor.line);
        cursor.line--;
        cursor.col = newCol;
        await this.pluginManager.emit("buffer:change", { buffer: this.buffer });
      }
      return;
    }

    if (key === "\r") {
      this.buffer.insert(cursor.line, cursor.col, "\n");
      cursor.line++;
      cursor.col = 0;
      await this.pluginManager.emit("buffer:change", { buffer: this.buffer });
      return;
    }

    this.buffer.insert(cursor.line, cursor.col, key);
    cursor.col++;
    await this.pluginManager.emit("buffer:change", { buffer: this.buffer });
  }

  private async handleCommand(key: string): Promise<void> {
    if (key === "\r") {
      const cmd = this.commandBuffer.trim();
      this.commandBuffer = "";
      const prevMode = this.state.mode;
      this.state.mode = "normal";
      await this.pluginManager.emit("mode:change", { from: prevMode, to: "normal" });

      if (cmd === "q") {
        this.running = false;
      } else if (cmd === "w") {
        try {
          await this.buffer.saveFile();
          if (this.state.filePath) {
            await this.pluginManager.emit("buffer:save", { filePath: this.state.filePath });
          }
        } catch (err) {
          process.stderr.write(`[editor] Save failed: ${err}\n`);
        }
      } else if (cmd === "wq") {
        try {
          await this.buffer.saveFile();
          if (this.state.filePath) {
            await this.pluginManager.emit("buffer:save", { filePath: this.state.filePath });
          }
          this.running = false;
        } catch (err) {
          process.stderr.write(`[editor] Save failed: ${err}\n`);
        }
      } else {
        const found = await this.pluginManager.commands.execute(cmd);
        if (!found) {
          process.stderr.write(`[editor] Unknown command: ${cmd}\n`);
        }
      }
      return;
    }

    if (key === "\x1b") {
      const prevMode = this.state.mode;
      this.commandBuffer = "";
      this.state.mode = "normal";
      await this.pluginManager.emit("mode:change", { from: prevMode, to: "normal" });
      return;
    }

    if (key === "\x7f") {
      this.commandBuffer = this.commandBuffer.slice(0, -1);
      process.stdout.write("\b \b");
      return;
    }

    this.commandBuffer += key;
    process.stdout.write(key);
  }

  private cleanup(): void {
    if (this.cleaned) return;
    this.cleaned = true;
    this.terminal.showCursor();
    this.terminal.disableRawMode();
    this.terminal.clearScreen();
    this.terminal.moveCursor(1, 1);
  }
}