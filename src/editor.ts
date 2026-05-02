import { Terminal } from "./terminal";
import { TextBuffer } from "./buffer";
import { Renderer } from "./renderer";
import type { EditorState, Mode } from "./types";

export class Editor {
  private terminal = new Terminal();
  private buffer = new TextBuffer();
  private renderer = new Renderer(this.terminal);
  private state: EditorState = {
    mode: "normal",
    cursor: { line: 1, col: 0 },
    filePath: null,
    dirty: false,
  };
  private running = false;
  private commandBuffer = "";

  async run(filePath?: string): Promise<void> {
    if (filePath) {
      await this.buffer.loadFile(filePath);
      this.state.filePath = filePath;
    }

    this.terminal.enableRawMode();
    this.terminal.hideCursor();
    this.running = true;

    process.on("exit", () => this.cleanup());
    process.on("SIGINT", () => { this.cleanup(); process.exit(0); });

    try {
      while (this.running) {
        this.state.dirty = this.buffer.dirty;
        this.renderer.render(this.buffer, this.state);
        this.terminal.showCursor();

        const key = await this.terminal.readKey();
        this.handleKey(key);
      }
    } finally {
      this.cleanup();
    }
  }

  private handleKey(key: string): void {
    switch (this.state.mode) {
      case "normal": this.handleNormal(key); break;
      case "insert": this.handleInsert(key); break;
      case "command": this.handleCommand(key); break;
    }
  }

  private handleNormal(key: string): void {
    const { cursor } = this.state;
    const lineCount = this.buffer.lineCount;

    switch (key) {
      case "h": cursor.col = Math.max(0, cursor.col - 1); break;
      case "l": cursor.col++; break;
      case "j": cursor.line = Math.min(lineCount, cursor.line + 1); break;
      case "k": cursor.line = Math.max(1, cursor.line - 1); break;
      case "i": this.state.mode = "insert"; break;
      case ":":
        this.terminal.moveCursor(this.terminal.rows, 1);
        process.stdout.write(":");
        this.commandBuffer = "";
        this.state.mode = "command";
        break;
    }
  }

  private handleInsert(key: string): void {
    const { cursor } = this.state;

    if (key === "\x1b") {
      this.state.mode = "normal";
      cursor.col = Math.max(0, cursor.col - 1);
      return;
    }

    if (key === "\x7f" || key === "\b") {
      if (cursor.col > 0) {
        this.buffer.delete(cursor.line, cursor.col - 1);
        cursor.col--;
      }
      return;
    }

    if (key === "\r") {
      this.buffer.insert(cursor.line, cursor.col, "\n");
      cursor.line++;
      cursor.col = 0;
      return;
    }

    this.buffer.insert(cursor.line, cursor.col, key);
    cursor.col++;
  }

  private handleCommand(key: string): void {
    if (key === "\r") {
      const cmd = this.commandBuffer.trim();
      this.commandBuffer = "";
      this.state.mode = "normal";

      if (cmd === "q") {
        this.running = false;
      } else if (cmd === "w") {
        this.buffer.saveFile().catch(console.error);
      } else if (cmd === "wq") {
        this.buffer.saveFile().then(() => { this.running = false; }).catch(console.error);
      }
      return;
    }

    if (key === "\x1b") {
      this.commandBuffer = "";
      this.state.mode = "normal";
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
    this.terminal.showCursor();
    this.terminal.disableRawMode();
    this.terminal.clearScreen();
    this.terminal.moveCursor(1, 1);
  }
}