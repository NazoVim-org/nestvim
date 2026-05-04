import { join } from "path";
import { Language, Parser } from "web-tree-sitter";
import type { LanguageId } from "../types";

// node_modules 内の WASM ファイルの相対パス
const WASM_PATHS: Partial<Record<LanguageId, string>> = {
  typescript: "tree-sitter-typescript/tree-sitter-typescript.wasm",
  python: "tree-sitter-python/tree-sitter-python.wasm",
  rust: "tree-sitter-rust/tree-sitter-rust.wasm",
  go: "tree-sitter-go/tree-sitter-go.wasm",
  c: "tree-sitter-c/tree-sitter-c.wasm",
  html: "tree-sitter-html/tree-sitter-html.wasm",
  css: "tree-sitter-css/tree-sitter-css.wasm",
  json: "tree-sitter-json/tree-sitter-json.wasm",
  lua: "@tree-sitter-grammars/tree-sitter-lua/tree-sitter-lua.wasm",
  markdown: "@tree-sitter-grammars/tree-sitter-markdown/tree-sitter-markdown.wasm",
};

const parserCache = new Map<LanguageId, Parser>();
const loadingPromises = new Map<LanguageId, Promise<Parser | null>>();
let initialized = false;

export async function initParser(): Promise<void> {
  // Parser.init() is idempotent, but guard against redundant checks
  if (initialized) return;
  await Parser.init();
  initialized = true;
}

/**
 * 指定言語の Parser を返す。初回はロードしてキャッシュする。
 * unknown または WASM のロードに失敗した場合は null を返す。
 */
export async function getParser(langId: LanguageId): Promise<Parser | null> {
  await initParser();

  if (langId === "unknown") return null;
  if (parserCache.has(langId)) return parserCache.get(langId)!;

  if (loadingPromises.has(langId)) {
    return loadingPromises.get(langId)!;
  }

  const wasmRelPath = WASM_PATHS[langId];
  if (!wasmRelPath) return null;

  const loadPromise = (async () => {
    try {
      const wasmPath = join(process.cwd(), "node_modules", wasmRelPath);
      const lang = await Language.load(wasmPath);
      const parser = new Parser();
      parser.setLanguage(lang);
      parserCache.set(langId, parser);
      return parser;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      process.stderr.write(`[highlight] Failed to load language "${langId}": ${message}\n`);
      return null;
    } finally {
      loadingPromises.delete(langId);
    }
  })();

  loadingPromises.set(langId, loadPromise);
  return loadPromise;
}
