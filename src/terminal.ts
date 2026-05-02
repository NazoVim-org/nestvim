export class Terminal {
  private _rows: number = 24;
  private _cols: number = 80;

  get rows() { return this._rows; }
  get cols() { return this._cols; }

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

  hideCursor(): void {
    process.stdout.write("\x1b[?25l");
  }

  showCursor(): void {
    process.stdout.write("\x1b[?25h");
  }

  writeLine(row: number, content: string): void {
    this.moveCursor(row, 1);
    const padded = content.slice(0, this._cols).padEnd(this._cols, " ");
    process.stdout.write(padded);
  }

  readKey(): Promise<string> {
    return new Promise((resolve) => {
      process.stdin.once("data", (key: string) => {
        resolve(key);
      });
    });
  }
}