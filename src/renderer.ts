import { Terminal } from "./terminal";
import { TextBuffer } from "./buffer";
import type { EditorState } from "./types";

export class Renderer {
  private scrollTop = 1;

  constructor(private terminal: Terminal) {}

  resetScroll(): void {
    this.scrollTop = 1;
  }

  render(buffer: TextBuffer, state: EditorState): void {
    this.terminal.moveHome();
    const visibleRows = this.terminal.rows - 2;

    // カーソルが画面外に出たらスクロール
    if (state.cursor.line < this.scrollTop) {
      this.scrollTop = state.cursor.line;
    } else if (state.cursor.line >= this.scrollTop + visibleRows) {
      this.scrollTop = state.cursor.line - visibleRows + 1;
    }

    for (let i = 0; i < visibleRows; i++) {
      const bufLine = this.scrollTop + i;
      const line = buffer.getLine(bufLine);
      this.terminal.writeLine(i + 1, line !== "" ? line : (bufLine <= buffer.lineCount ? "" : "~"));
    }

    const status = `-- ${state.mode.toUpperCase()} -- ${state.filePath ?? "[No Name]"}${state.dirty ? " [+]" : ""}`;
    this.terminal.writeLine(this.terminal.rows - 1, status);

    const screenRow = state.cursor.line - this.scrollTop + 1;
    this.terminal.moveCursor(screenRow, state.cursor.col + 1);
  }
}