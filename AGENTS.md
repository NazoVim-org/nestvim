# AGENT_TASK: nazoedit — 初期スキャフォールド

## 概要

Bun + TypeScript 製の TUI テキストエディタ `nazoedit` のリポジトリを初期構築する。
ターミナルを raw mode で制御し、Vim ライクなモーダル編集（Normal / Insert）が動く
最小限のエディタをフェーズ 1 として実装する。

---

## タスク一覧

### Task 1 — 🔴 [Critical] リポジトリ・パッケージ初期化

**問題**
プロジェクトが存在しないため、まず Bun プロジェクトとして初期化する必要がある。

**対象ファイル**
| ファイル | 変更内容 |
|----------|----------|
| `package.json` | 新規作成 |
| `tsconfig.json` | 新規作成 |
| `.gitignore` | 新規作成 |
| `README.md` | 新規作成（最小限） |

**修正内容**

```bash
# 実行コマンド
bun init -y
```

`package.json` を以下の内容に上書きする：

```json
{
  "name": "nazoedit",
  "version": "0.1.0",
  "description": "A minimal Vim-like TUI editor",
  "main": "src/main.ts",
  "scripts": {
    "start": "bun run src/main.ts",
    "dev": "bun --watch src/main.ts"
  },
  "dependencies": {
    "@codemirror/state": "^6.0.0"
  },
  "devDependencies": {
    "@types/bun": "latest"
  }
}
```

`tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "types": ["bun-types"]
  }
}
```

`.gitignore`:

```
node_modules/
dist/
.DS_Store
```

**完了条件**
- `ls package.json tsconfig.json .gitignore` がすべて存在すること
- `bun install` がエラーなく完了すること
- `bun run start` を実行して「nazoedit starting...」が出力されること（Task 5 完了後）

---

### Task 2 — 🔴 [Critical] ディレクトリ構造の作成

**問題**
モジュール分割のベースとなるディレクトリとファイルを作成する。

**対象ファイル**
| ファイル | 変更内容 |
|----------|----------|
| `src/main.ts` | エントリーポイント（新規） |
| `src/terminal.ts` | ターミナル制御（新規） |
| `src/buffer.ts` | テキストバッファ（新規） |
| `src/editor.ts` | モード管理・イベントループ（新規） |
| `src/renderer.ts` | 画面描画（新規） |
| `src/types.ts` | 共通型定義（新規） |

**修正内容**

`src/types.ts` — 共通型定義:

```typescript
// src/types.ts

export type Mode = "normal" | "insert" | "command";

export interface Position {
  line: number;
  col: number;
}

export interface Size {
  rows: number;
  cols: number;
}

export interface EditorState {
  mode: Mode;
  cursor: Position;
  filePath: string | null;
  dirty: boolean;
}
```

`src/main.ts` — エントリーポイント（スタブ）:

```typescript
// src/main.ts
import { Editor } from "./editor";

async function main() {
  console.log("nazoedit starting...");
  const editor = new Editor();
  await editor.run();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

残りのファイル（`terminal.ts`, `buffer.ts`, `editor.ts`, `renderer.ts`）は
空のクラス or 関数スタブとして作成する（Task 3〜5 で実装する）。

**完了条件**
- `find src -name "*.ts" | sort` の出力に 6 ファイルすべてが含まれること
- `bun run src/main.ts` がクラッシュせず「nazoedit starting...」を出力して終了すること

---

### Task 3 — 🔴 [Critical] terminal.ts — raw mode とターミナル制御

**問題**
エディタの基盤となるターミナル制御モジュール。
`process.stdin` を raw mode にして1文字ずつ入力を受け取る。
ANSI エスケープコードでカーソル移動・画面クリアを行う。

**対象ファイル**
| ファイル | 変更内容 |
|----------|----------|
| `src/terminal.ts` | 実装 |

**修正内容**

```typescript
// src/terminal.ts

export class Terminal {
  private _rows: number = 24;
  private _cols: number = 80;

  get rows() { return this._rows; }
  get cols() { return this._cols; }

  /** ターミナルを raw mode にする */
  enableRawMode(): void {
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.setEncoding("utf8");
    this.updateSize();
    process.on("SIGWINCH", () => this.updateSize());
  }

  /** raw mode を解除してターミナルを元に戻す */
  disableRawMode(): void {
    process.stdin.setRawMode(false);
    process.stdin.pause();
  }

  private updateSize(): void {
    this._rows = process.stdout.rows ?? 24;
    this._cols = process.stdout.columns ?? 80;
  }

  /** カーソルを指定位置に移動（1-indexed） */
  moveCursor(row: number, col: number): void {
    process.stdout.write(`\x1b[${row};${col}H`);
  }

  /** 画面全体をクリア */
  clearScreen(): void {
    process.stdout.write("\x1b[2J");
  }

  /** カーソルを非表示 */
  hideCursor(): void {
    process.stdout.write("\x1b[?25l");
  }

  /** カーソルを表示 */
  showCursor(): void {
    process.stdout.write("\x1b[?25h");
  }

  /** 1行書き込む（末尾の空白でクリア） */
  writeLine(row: number, content: string): void {
    this.moveCursor(row, 1);
    const padded = content.slice(0, this._cols).padEnd(this._cols, " ");
    process.stdout.write(padded);
  }

  /** 1文字入力を非同期で読む */
  readKey(): Promise<string> {
    return new Promise((resolve) => {
      process.stdin.once("data", (key: string) => {
        resolve(key);
      });
    });
  }
}
```

**完了条件**
- `Terminal` クラスが import できること
- `new Terminal()` がエラーを投げないこと

---

### Task 4 — 🔴 [Critical] buffer.ts — テキストバッファ

**問題**
`@codemirror/state` の `Text` を使ってテキストを管理する。
行の取得・挿入・削除の基本操作を提供する。

**対象ファイル**
| ファイル | 変更内容 |
|----------|----------|
| `src/buffer.ts` | 実装 |

**修正内容**

```typescript
// src/buffer.ts
import { Text } from "@codemirror/state";

export class Buffer {
  private doc: Text;
  public filePath: string | null = null;
  public dirty: boolean = false;

  constructor(content: string = "") {
    this.doc = Text.of(content.split("\n"));
  }

  get lineCount(): number {
    return this.doc.lines;
  }

  /** 1-indexed で行のテキストを返す */
  getLine(lineNumber: number): string {
    if (lineNumber < 1 || lineNumber > this.doc.lines) return "";
    return this.doc.line(lineNumber).text;
  }

  /** 全テキストを返す */
  toString(): string {
    return this.doc.toString();
  }

  /** 指定位置に文字を挿入する */
  insert(line: number, col: number, text: string): void {
    const lineInfo = this.doc.line(line);
    const pos = lineInfo.from + col;
    const before = this.doc.sliceString(0, pos);
    const after = this.doc.sliceString(pos);
    this.doc = Text.of((before + text + after).split("\n"));
    this.dirty = true;
  }

  /** 指定位置の1文字を削除する */
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

  /** ファイルから読み込む */
  async loadFile(path: string): Promise<void> {
    const file = Bun.file(path);
    const content = await file.text();
    this.doc = Text.of(content.split("\n"));
    this.filePath = path;
    this.dirty = false;
  }

  /** ファイルに保存する */
  async saveFile(): Promise<void> {
    if (!this.filePath) throw new Error("No file path set");
    await Bun.write(this.filePath, this.toString());
    this.dirty = false;
  }
}
```

**完了条件**
- `bun run -e "import { Buffer } from './src/buffer'; const b = new Buffer('hello'); console.log(b.getLine(1))"` が `hello` を出力すること

---

### Task 5 — 🟠 [Medium] editor.ts + renderer.ts — イベントループと描画

**問題**
`Editor` クラスがイベントループを持ち、キー入力を受けてモードを切り替える。
`Renderer` が `Buffer` の内容を `Terminal` に描画する。

**対象ファイル**
| ファイル | 変更内容 |
|----------|----------|
| `src/editor.ts` | 実装 |
| `src/renderer.ts` | 実装 |

**修正内容**

`src/renderer.ts`:

```typescript
// src/renderer.ts
import { Terminal } from "./terminal";
import { Buffer } from "./buffer";
import type { EditorState } from "./types";

export class Renderer {
  constructor(private terminal: Terminal) {}

  render(buffer: Buffer, state: EditorState): void {
    this.terminal.clearScreen();

    const visibleRows = this.terminal.rows - 2; // ステータスバー分を引く

    for (let i = 1; i <= visibleRows; i++) {
      const line = buffer.getLine(i);
      this.terminal.writeLine(i, line ?? "~");
    }

    // ステータスバー
    const status = `-- ${state.mode.toUpperCase()} -- ${state.filePath ?? "[No Name]"}${state.dirty ? " [+]" : ""}`;
    this.terminal.writeLine(this.terminal.rows - 1, status);

    // カーソルを正しい位置に
    this.terminal.moveCursor(state.cursor.line, state.cursor.col + 1);
  }
}
```

`src/editor.ts`:

```typescript
// src/editor.ts
import { Terminal } from "./terminal";
import { Buffer } from "./buffer";
import { Renderer } from "./renderer";
import type { EditorState, Mode } from "./types";

export class Editor {
  private terminal = new Terminal();
  private buffer = new Buffer();
  private renderer = new Renderer(this.terminal);
  private state: EditorState = {
    mode: "normal",
    cursor: { line: 1, col: 0 },
    filePath: null,
    dirty: false,
  };
  private running = false;
  private commandBuffer = "";

  async run(filePath?: string): Promise<void> {
    if (filePath) {
      await this.buffer.loadFile(filePath);
      this.state.filePath = filePath;
    }

    this.terminal.enableRawMode();
    this.terminal.hideCursor();
    this.running = true;

    // クリーンアップ
    process.on("exit", () => this.cleanup());
    process.on("SIGINT", () => { this.cleanup(); process.exit(0); });

    try {
      while (this.running) {
        this.state.dirty = this.buffer.dirty;
        this.renderer.render(this.buffer, this.state);
        this.terminal.showCursor();

        const key = await this.terminal.readKey();
        this.handleKey(key);
      }
    } finally {
      this.cleanup();
    }
  }

  private handleKey(key: string): void {
    switch (this.state.mode) {
      case "normal": this.handleNormal(key); break;
      case "insert": this.handleInsert(key); break;
    }
  }

  private handleNormal(key: string): void {
    const { cursor } = this.state;
    const lineCount = this.buffer.lineCount;

    switch (key) {
      case "h": cursor.col = Math.max(0, cursor.col - 1); break;
      case "l": cursor.col++; break;
      case "j": cursor.line = Math.min(lineCount, cursor.line + 1); break;
      case "k": cursor.line = Math.max(1, cursor.line - 1); break;
      case "i": this.state.mode = "insert"; break;
      case ":":
        // コマンドモードは簡易実装
        this.terminal.moveCursor(this.terminal.rows, 1);
        process.stdout.write(":");
        this.commandBuffer = "";
        this.state.mode = "command";
        break;
    }
  }

  private handleInsert(key: string): void {
    const { cursor } = this.state;

    if (key === "\x1b") { // ESC
      this.state.mode = "normal";
      cursor.col = Math.max(0, cursor.col - 1);
      return;
    }

    if (key === "\x7f" || key === "\b") { // Backspace
      if (cursor.col > 0) {
        this.buffer.delete(cursor.line, cursor.col - 1);
        cursor.col--;
      }
      return;
    }

    if (key === "\r") { // Enter
      this.buffer.insert(cursor.line, cursor.col, "\n");
      cursor.line++;
      cursor.col = 0;
      return;
    }

    // 通常文字
    this.buffer.insert(cursor.line, cursor.col, key);
    cursor.col++;
  }

  private cleanup(): void {
    this.terminal.showCursor();
    this.terminal.disableRawMode();
    this.terminal.clearScreen();
    this.terminal.moveCursor(1, 1);
  }
}
```

**完了条件**
- `bun run src/main.ts` を実行してターミナルが raw mode になりカーソルが表示されること
- `hjkl` でカーソルが動くこと
- `i` で Insert mode に入り文字が打てること
- `ESC` で Normal mode に戻ること
- `:q` + Enter でエディタが終了すること

---

### Task 6 — 🟡 [Minor] コマンドモードの実装（`:w` / `:q` / `:wq`）

**問題**
Task 5 のコマンドモードはスタブ状態。`:w`・`:q`・`:wq` を実装する。

**対象ファイル**
| ファイル | 変更内容 |
|----------|----------|
| `src/editor.ts` | `handleCommand` メソッドを追加 |

**修正内容**

`handleKey` に `"command"` ケースを追加し、`handleCommand` を実装する：

```typescript
// editor.ts に追加

case "command": this.handleCommand(key); break;

// ---

private handleCommand(key: string): void {
  if (key === "\r") { // Enter — コマンド確定
    const cmd = this.commandBuffer.trim();
    this.commandBuffer = "";
    this.state.mode = "normal";

    if (cmd === "q") {
      this.running = false;
    } else if (cmd === "w") {
      this.buffer.saveFile().catch(console.error);
    } else if (cmd === "wq") {
      this.buffer.saveFile().then(() => { this.running = false; }).catch(console.error);
    }
    return;
  }

  if (key === "\x1b") { // ESC — キャンセル
    this.commandBuffer = "";
    this.state.mode = "normal";
    return;
  }

  if (key === "\x7f") { // Backspace
    this.commandBuffer = this.commandBuffer.slice(0, -1);
    process.stdout.write("\b \b");
    return;
  }

  this.commandBuffer += key;
  process.stdout.write(key);
}
```

**完了条件**
- `:q` + Enter でエディタが終了すること
- `:w` + Enter でファイルが保存されること（`cat <ファイル名>` で確認）
- `:wq` + Enter で保存して終了すること
- `ESC` でコマンドキャンセルして Normal mode に戻ること

---

## 作業順序

```
Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6
```

Task 1〜4 は依存関係があるため順番通りに実行すること。
Task 5 は Task 3・4 が完了していないと動かない。
Task 6 は Task 5 の後に実行する。

## コミットメッセージ規約

Conventional Commits を使う。

```
feat: initialize bun project and directory structure
feat(terminal): implement raw mode and ANSI escape control
feat(buffer): implement text buffer using @codemirror/state
feat(editor): implement event loop, Normal/Insert mode
feat(editor): implement command mode (:w :q :wq)
```

## 注意事項

- `process.stdin.setRawMode` は Bun で動作する（Node.js と同じ API）
- `@codemirror/state` の `Text` の行番号は **1-indexed**
- `Terminal.writeLine` は行全体を上書きするため、前の描画が残ることはない
- `Buffer` クラスの名前が `globalThis.Buffer`（Node.js の Buffer）と衝突する場合は `TextBuffer` に改名すること
- `process.on("exit")` と `SIGINT` の両方でクリーンアップを呼ぶことで、`Ctrl+C` でもターミナルが壊れないようにしている
- Task 5 完了後、実際にターミナルで起動して動作確認を必ず行うこと（CI がないため手動確認が唯一のテスト手段）
