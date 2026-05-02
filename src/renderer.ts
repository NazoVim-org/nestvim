import { Terminal } from "./terminal";
import { TextBuffer } from "./buffer";
import type { EditorState } from "./types";

export class Renderer {
  constructor(private terminal: Terminal) {}

  render(buffer: TextBuffer, state: EditorState): void {
    this.terminal.moveHome();

    const visibleRows = this.terminal.rows - 2;

    for (let i = 1; i <= visibleRows; i++) {
      const line = buffer.getLine(i);
      this.terminal.writeLine(i, line || "~");
    }

    const status = `-- ${state.mode.toUpperCase()} -- ${state.filePath ?? "[No Name]"}${state.dirty ? " [+]" : ""}`;
    this.terminal.writeLine(this.terminal.rows - 1, status);

    this.terminal.moveCursor(state.cursor.line, state.cursor.col + 1);
  }
}