export type Mode = "normal" | "insert" | "command";

export interface Position {
  line: number;
  col: number;
}

export interface EditorState {
  mode: Mode;
  cursor: Position;
  filePath: string | null;
  dirty: boolean;
}