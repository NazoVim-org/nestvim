import type { PluginAPI, EditorEventName, EditorEventPayload } from "./types";
import type { TextBuffer } from "../buffer";
import type { EditorState } from "../types";
import { EventEmitter } from "./events";
import { CommandRegistry } from "./commands";
import { KeymapRegistry } from "./keymaps";

export class PluginAPIImpl implements PluginAPI {
  constructor(
    private emitter: EventEmitter,
    private commands: CommandRegistry,
    private keymaps: KeymapRegistry,
    private getBufferFn: () => TextBuffer,
    private getStateFn: () => EditorState
  ) {}

  on<T extends EditorEventName>(
    event: T,
    handler: (payload: EditorEventPayload<T>) => void
  ): void {
    this.emitter.on(event, handler);
  }

  off<T extends EditorEventName>(
    event: T,
    handler: (payload: EditorEventPayload<T>) => void
  ): void {
    this.emitter.off(event, handler);
  }

  addCommand(name: string, handler: () => void | Promise<void>): void {
    this.commands.register(name, handler);
  }

  addKeymap(
    mode: EditorState["mode"],
    key: string,
    handler: () => void | Promise<void>
  ): void {
    this.keymaps.register(mode, key, handler);
  }

  getBuffer(): Readonly<TextBuffer> {
    return this.getBufferFn();
  }

  getState(): Readonly<EditorState> {
    return this.getStateFn();
  }

  log(message: string): void {
    process.stderr.write(`[plugin] ${message}\n`);
  }
}