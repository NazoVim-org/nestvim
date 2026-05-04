import type { EditorState } from "../types";

type KeymapHandler = () => void | Promise<void>;
type ModeKeymaps = Map<string, KeymapHandler>;

export class KeymapRegistry {
  private keymaps = new Map<EditorState["mode"], ModeKeymaps>();

  register(mode: EditorState["mode"], key: string, handler: KeymapHandler): void {
    if (!this.keymaps.has(mode)) {
      this.keymaps.set(mode, new Map());
    }
    this.keymaps.get(mode)?.set(key, handler);
  }

  async handle(mode: EditorState["mode"], key: string): Promise<boolean> {
    const handler = this.keymaps.get(mode)?.get(key);
    if (!handler) return false;
    await handler();
    return true;
  }
}
