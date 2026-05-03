import type { LanguageId } from "./types";

const EXT_MAP: Record<string, LanguageId> = {
  ".ts":   "typescript",
  ".tsx":  "typescript",
  ".js":   "typescript",  // tree-sitter-typescript WASM handles JavaScript syntax
  ".jsx":  "typescript",  // tree-sitter-typescript WASM handles JavaScript syntax
  ".mjs":  "typescript",  // tree-sitter-typescript WASM handles JavaScript syntax
  ".cjs":  "typescript",  // tree-sitter-typescript WASM handles JavaScript syntax
  ".py":   "python",
  ".rs":   "rust",
  ".go":   "go",
  ".c":    "c",
  ".h":    "c",
  ".cpp":  "c",  // No C++ WASM available; fallback to C grammar
  ".cc":   "c",  // No C++ WASM available; fallback to C grammar
  ".html": "html",
  ".htm":  "html",
  ".css":  "css",
  ".json": "json",
  ".md":   "markdown",
  ".lua":  "lua",
};

export function detectLanguage(filePath: string | null): LanguageId {
  if (!filePath) return "unknown";
  const ext = filePath.match(/\.[^.]+$/)?.[0]?.toLowerCase() ?? "";
  return EXT_MAP[ext] ?? "unknown";
}
