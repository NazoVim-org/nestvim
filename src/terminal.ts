function displayWidth(str: string): number {
  let width = 0;
  for (const char of str) {
    const cp = char.codePointAt(0) ?? 0;
    if (
      (cp >= 0x1100 && cp <= 0x115f) ||
      (cp >= 0x2e80 && cp <= 0x303e) ||
      (cp >= 0x3040 && cp <= 0xa4cf) ||
      (cp >= 0xac00 && cp <= 0xd7a3) ||
      (cp >= 0xf900 && cp <= 0xfaff) ||
      (cp >= 0xfe10 && cp <= 0xfe1f) ||
      (cp >= 0xfe30 && cp <= 0xfe4f) ||
      (cp >= 0xff00 && cp <= 0xff60) ||
      (cp >= 0xffe0 && cp <= 0xffe6) ||
      (cp >= 0x1f300 && cp <= 0x1faff)
    ) {
      width += 2;
    } else {
      width += 1;
    }
  }
  return width;
}

function stripAnsi(str: string): string {
  // eslint-disable-next-line no-control-regex
  return str.replace(/\x1b\[[0-9;]*m/g, "");
}

export class Terminal {
  private _rows: number = 24;
  private _cols: number = 80;

  get rows() {
    return this._rows;
  }
  get cols() {
    return this._cols;
  }

  enableRawMode(): void {
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.setEncoding("utf8");
    this.updateSize();
    process.on("SIGWINCH", () => this.updateSize());
  }

  disableRawMode(): void {
    process.stdin.setRawMode(false);
    process.stdin.pause();
  }

  private updateSize(): void {
    this._rows = process.stdout.rows ?? 24;
    this._cols = process.stdout.columns ?? 80;
  }

  moveCursor(row: number, col: number): void {
    process.stdout.write(`\x1b[${row};${col}H`);
  }

  clearScreen(): void {
    process.stdout.write("\x1b[2J");
  }

  moveHome(): void {
    process.stdout.write("\x1b[H");
  }

  hideCursor(): void {
    process.stdout.write("\x1b[?25l");
  }

  showCursor(): void {
    process.stdout.write("\x1b[?25h");
  }

  writeLine(row: number, content: string): void {
    this.moveCursor(row, 1);

    // content（ANSI込み）をそのまま出力してパディング
    const visibleContent = this.truncateToWidth(content, this._cols);
    const plainVisible = stripAnsi(visibleContent);
    let plainW = 0;
    for (const char of plainVisible) {
      plainW += displayWidth(char);
    }

    process.stdout.write(visibleContent + " ".repeat(Math.max(0, this._cols - plainW)));
  }

  private truncateToWidth(str: string, maxWidth: number): string {
    let result = "";
    let w = 0;
    let i = 0;
    let inAnsi = false;

    while (i < str.length) {
      if (!inAnsi && str[i] === "\x1b" && str[i + 1] === "[") {
        inAnsi = true;
        result += str[i];
        i++;
      } else if (inAnsi) {
        result += str[i];
        if (str[i] === "m") inAnsi = false;
        i++;
      } else {
        const codePoint = str.codePointAt(i);
        if (codePoint === undefined) break;
        const char = String.fromCodePoint(codePoint);
        const cw = displayWidth(char);
        if (w + cw > maxWidth) break;
        result += char;
        w += cw;
        i += codePoint > 0xffff ? 2 : 1;
      }
    }
    return result;
  }

  readKey(): Promise<string> {
    return new Promise((resolve) => {
      process.stdin.once("data", (key: string) => {
        resolve(key);
      });
    });
  }
}
