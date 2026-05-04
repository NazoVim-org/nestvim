import { Text } from "@codemirror/state";

export class TextBuffer {
  private doc: Text;
  public filePath: string | null = null;
  public dirty: boolean = false;

  constructor(content: string = "") {
    this.doc = Text.of(content.split("\n"));
  }

  get lineCount(): number {
    return this.doc.lines;
  }

  getLine(lineNumber: number): string {
    if (lineNumber < 1 || lineNumber > this.doc.lines) return "";
    return this.doc.line(lineNumber).text;
  }

  toString(): string {
    return this.doc.toString();
  }

  insert(line: number, col: number, text: string): void {
    const lineInfo = this.doc.line(line);
    const pos = lineInfo.from + col;
    const before = this.doc.sliceString(0, pos);
    const after = this.doc.sliceString(pos);
    this.doc = Text.of((before + text + after).split("\n"));
    this.dirty = true;
  }

  delete(line: number, col: number): void {
    if (col < 0) return;
    const lineInfo = this.doc.line(line);
    const pos = lineInfo.from + col;
    if (pos >= this.doc.length) return;
    const before = this.doc.sliceString(0, pos);
    const after = this.doc.sliceString(pos + 1);
    this.doc = Text.of((before + after).split("\n"));
    this.dirty = true;
  }

  async loadFile(path: string): Promise<void> {
    const file = Bun.file(path);
    const content = await file.text();
    const newDoc = Text.of(content.split("\n"));
    this.doc = newDoc;
    this.filePath = path;
    this.dirty = false;
  }

  async saveFile(): Promise<void> {
    if (!this.filePath) throw new Error("No file path set");
    try {
      await Bun.write(this.filePath, this.toString());
      this.dirty = false;
    } catch (e) {
      this.dirty = true;
      throw e;
    }
  }

  mergeWithPrevLine(line: number): number {
    if (line <= 1) return 0;
    const prevLine = this.doc.line(line - 1);
    const curLine = this.doc.line(line);
    const mergedCol = prevLine.text.length;
    const before = this.doc.sliceString(0, prevLine.to);
    const after = this.doc.sliceString(curLine.from);
    this.doc = Text.of((before + after).split("\n"));
    this.dirty = true;
    return mergedCol;
  }
}
