import type { TextBuffer } from "../buffer";
import type { EditorState } from "../types";

export type EditorEventMap = {
  "editor:ready": undefined;
  "mode:change": { from: EditorState["mode"]; to: EditorState["mode"] };
  "buffer:change": { buffer: TextBuffer };
  "buffer:save": { filePath: string };
  "key:normal": { key: string };
  "key:insert": { key: string };
};

export type EditorEventName = keyof EditorEventMap;
export type EditorEventPayload<T extends EditorEventName> = EditorEventMap[T];

export interface PluginAPI {
  on<T extends EditorEventName>(event: T, handler: (payload: EditorEventPayload<T>) => void): void;

  off<T extends EditorEventName>(event: T, handler: (payload: EditorEventPayload<T>) => void): void;

  addCommand(name: string, handler: () => void | Promise<void>): void;

  addKeymap(mode: EditorState["mode"], key: string, handler: () => void | Promise<void>): void;

  getBuffer(): Readonly<TextBuffer>;

  getState(): Readonly<EditorState>;

  log(message: string): void;
}

export interface PluginDefinition {
  name: string;
  version?: string;
  setup(api: PluginAPI): void | Promise<void>;
}

export interface LoadedPlugin {
  name: string;
  type: "typescript" | "lua";
  definition: PluginDefinition;
}
