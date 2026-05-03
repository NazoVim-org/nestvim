/** 1行内の色付きスパン */
export interface HighlightSpan {
  /** 開始カラム（0-indexed, 包含） */
  start: number;
  /** 終了カラム（0-indexed, 排他） */
  end: number;
  /** ANSI エスケープコード（例: "\x1b[33m"） */
  ansi: string;
}

/** 1行分のハイライト情報 */
export type HighlightedLine = HighlightSpan[];

/** ドキュメント全体のハイライト情報（1-indexed） */
export type HighlightedDoc = Map<number, HighlightedLine>;

/** 言語識別子 */
export type LanguageId =
  | "typescript"
  | "python"
  | "rust"
  | "go"
  | "c"
  | "html"
  | "css"
  | "json"
  | "markdown"
  | "lua"
  | "unknown";
