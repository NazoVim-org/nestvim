import { readdir, readFile } from "fs/promises";
import { join, resolve } from "path";
import { LuaFactory } from "wasmoon";
import type { LoadedPlugin, PluginAPI, PluginDefinition } from "../types";

function bridgeAPI(lua: Awaited<ReturnType<LuaFactory["createEngine"]>>, api: PluginAPI): object {
  const bridged = {
    on: (event: string, handler: () => void) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      api.on(event as any, handler as any);
    },
    off: (event: string, handler: () => void) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      api.off(event as any, handler as any);
    },
    addCommand: (name: string, handler: () => void) => {
      api.addCommand(name, handler);
    },
    addKeymap: (mode: string, key: string, handler: () => void) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      api.addKeymap(mode as any, key, handler);
    },
    log: (message: string) => {
      api.log(message);
    },
  };
  lua.global.set("api", bridged);
  return bridged;
}

export async function loadLuaPlugins(pluginsDir: string, api: PluginAPI): Promise<LoadedPlugin[]> {
  const loaded: LoadedPlugin[] = [];
  const factory = new LuaFactory();

  let files: string[];
  try {
    files = await readdir(pluginsDir);
  } catch {
    return [];
  }

  const luaFiles = files.filter((f) => f.endsWith(".lua"));

  for (const file of luaFiles) {
    const filePath = resolve(join(pluginsDir, file));
    try {
      const source = await readFile(filePath, "utf8");
      const lua = await factory.createEngine();

      const bridged = bridgeAPI(lua, api);

      const result = await lua.doString(`return (function() ${source} end)()`);

      const name: string =
        typeof result === "object" && result !== null && "name" in result
          ? String(result.name)
          : file.replace(".lua", "");

      const setupFn: unknown =
        typeof result === "object" && result !== null && "setup" in result ? result.setup : null;

      if (typeof setupFn === "function") {
        await setupFn(bridged);
      }

      const definition: PluginDefinition = {
        name,
        setup: () => {},
      };

      loaded.push({ name, type: "lua", definition });
      process.stderr.write(`[plugin] Loaded Lua plugin: ${name}\n`);
    } catch (err) {
      process.stderr.write(
        `[PluginLoader] Failed to load "${file}": ${err instanceof Error ? err.message : err}\n`,
      );
    }
  }

  return loaded;
}
