import { Terminal } from "./terminal";
import { TextBuffer } from "./buffer";
import type { EditorState } from "./types";
import type { Highlighter } from "./highlight/highlighter";
import type { HighlightedLine } from "./highlight/types";
import { RESET } from "./highlight/theme";

export class Renderer {
  private scrollTop = 1;

  constructor(
    private terminal: Terminal,
    private highlighter?: Highlighter
  ) {}

  resetScroll(): void {
    this.scrollTop = 1;
  }

  render(buffer: TextBuffer, state: EditorState): void {
    this.terminal.moveHome();
    const visibleRows = this.terminal.rows - 2;

    if (state.cursor.line < this.scrollTop) {
      this.scrollTop = state.cursor.line;
    } else if (state.cursor.line >= this.scrollTop + visibleRows) {
      this.scrollTop = state.cursor.line - visibleRows + 1;
    }

    const hlCache = this.highlighter?.getCache();

    for (let i = 0; i < visibleRows; i++) {
      const bufLine = this.scrollTop + i;
      const rawLine = buffer.getLine(bufLine);
      const isTildeRow = rawLine === "" && bufLine > buffer.lineCount;
      const displayText = isTildeRow ? "~" : rawLine;

      const spans = hlCache?.get(bufLine);
      const colored = (spans && !isTildeRow)
        ? applySpans(displayText, spans)
        : displayText;

      this.terminal.writeLine(i + 1, colored);
    }

    const status = `-- ${state.mode.toUpperCase()} -- ${state.filePath ?? "[No Name]"}${state.dirty ? " [+]" : ""}`;
    this.terminal.writeLine(this.terminal.rows - 1, status);

    const screenRow = state.cursor.line - this.scrollTop + 1;
    this.terminal.moveCursor(screenRow, state.cursor.col + 1);
  }
}

/**
 * 1行のプレーンテキストにスパンの ANSI コードを適用して返す。
 */
function applySpans(line: string, spans: HighlightedLine): string {
  if (spans.length === 0) return line;

  const sorted = [...spans].sort((a, b) => a.start - b.start);
  const chars = [...line];
  let result = "";
  let col = 0;

  for (const span of sorted) {
    const start = span.start;
    const end = Math.min(
      span.end === Infinity ? chars.length : span.end,
      chars.length
    );

    if (col < start) {
      result += chars.slice(col, start).join("");
      col = start;
    }
    if (col < end) {
      result += span.ansi + chars.slice(col, end).join("") + RESET;
      col = end;
    }
  }

  if (col < chars.length) {
    result += chars.slice(col).join("");
  }

  return result;
}