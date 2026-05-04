import { getParser, initParser } from "./languages/index";
import { RESET, THEME } from "./theme";
import type { HighlightedDoc, LanguageId } from "./types";

export class Highlighter {
  private cache: HighlightedDoc = new Map();
  private dirty = true;
  private currentLang: LanguageId = "unknown";
  private failureCount = 0;
  private static readonly MAX_FAILURES = 3;

  async init(): Promise<void> {
    await initParser();
  }

  setLanguage(langId: LanguageId): void {
    if (this.currentLang !== langId) {
      this.currentLang = langId;
      this.dirty = true;
      this.failureCount = 0;
    }
  }

  markDirty(): void {
    this.dirty = true;
    this.failureCount = 0;
  }

  getCache(): HighlightedDoc {
    return this.cache;
  }

  /**
   * テキスト全体を再パースしてハイライトキャッシュを更新する。
   * dirty でない場合はスキップ。
   */
  async update(text: string): Promise<void> {
    if (!this.dirty) return;

    const parser = await getParser(this.currentLang);
    if (!parser) {
      this.cache.clear();
      this.dirty = false;
      return;
    }

    try {
      const tree = parser.parse(text);
      if (!tree) {
        this.dirty = false;
        this.failureCount = 0;
        return;
      }
      const newCache: HighlightedDoc = new Map();

      // BFS でツリーを走査してリーフノードを収集
      const queue: ReturnType<typeof tree.rootNode.child>[] = [tree.rootNode];
      while (queue.length > 0) {
        const node = queue.shift();
        if (!node) continue;

        if (node.childCount === 0) {
          const ansi = THEME[node.type] ?? null;
          if (ansi) {
            // ノードが複数行にまたがる場合は行ごとに分割
            const startLine = node.startPosition.row + 1; // 1-indexed
            const endLine = node.endPosition.row + 1;

            for (let lineNo = startLine; lineNo <= endLine; lineNo++) {
              const spanStart = lineNo === startLine ? node.startPosition.column : 0;
              const spanEnd = lineNo === endLine ? node.endPosition.column : Infinity;

              if (!newCache.has(lineNo)) newCache.set(lineNo, []);
              newCache.get(lineNo)!.push({ start: spanStart, end: spanEnd, ansi });
            }
          }
        } else {
          for (let i = 0; i < node.childCount; i++) {
            const child = node.child(i);
            if (child) queue.push(child);
          }
        }
      }

      this.cache = newCache;
      this.dirty = false;
      this.failureCount = 0;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      process.stderr.write(`[highlight] Failed to parse text: ${message}\n`);
      this.failureCount++;
      if (this.failureCount >= Highlighter.MAX_FAILURES) {
        this.dirty = false; // Give up after max failures
        this.failureCount = 0;
      }
    }
  }
}
