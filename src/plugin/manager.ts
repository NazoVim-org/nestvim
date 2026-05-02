import { join } from "path";
import type { TextBuffer } from "../buffer";
import type { EditorState } from "../types";
import type { EditorEventName, EditorEventPayload } from "./types";
import { EventEmitter } from "./events";
import { CommandRegistry } from "./commands";
import { KeymapRegistry } from "./keymaps";
import { PluginAPIImpl } from "./api";
import { loadTSPlugins } from "./loaders/typescript";
import { loadLuaPlugins } from "./loaders/lua";
import type { LoadedPlugin } from "./types";

export class PluginManager {
  private emitter = new EventEmitter();
  readonly commands = new CommandRegistry();
  readonly keymaps = new KeymapRegistry();
  private plugins: LoadedPlugin[] = [];
  private api: PluginAPIImpl;

  constructor(
    private getBuffer: () => TextBuffer,
    private getState: () => EditorState
  ) {
    this.api = new PluginAPIImpl(
      this.emitter,
      this.commands,
      this.keymaps,
      getBuffer,
      getState
    );
  }

  async loadAll(): Promise<void> {
    const pluginsDir = join(import.meta.dir, "..", "plugins");

    const [tsPlugins, luaPlugins] = await Promise.all([
      loadTSPlugins(pluginsDir),
      loadLuaPlugins(pluginsDir, this.api),
    ]);

    for (const plugin of tsPlugins) {
      try {
        await plugin.definition.setup(this.api);
        this.plugins.push(plugin);
      } catch (err) {
        process.stderr.write(`[PluginManager] setup() failed for "${plugin.name}": ${err instanceof Error ? err.message : err}\n`);
      }
    }

    this.plugins.push(...luaPlugins);

    process.stderr.write(
      `[plugin] ${this.plugins.length} plugin(s) loaded.\n`
    );
  }

  async emit<T extends EditorEventName>(
    event: T,
    payload: EditorEventPayload<T>
  ): Promise<void> {
    await this.emitter.emit(event, payload);
  }
}