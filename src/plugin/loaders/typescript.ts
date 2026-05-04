import { readdir } from "fs/promises";
import { join, resolve } from "path";
import type { LoadedPlugin, PluginDefinition } from "../types";

function isPluginDefinition(obj: unknown): obj is PluginDefinition {
  return (
    typeof obj === "object" &&
    obj !== null &&
    typeof (obj as PluginDefinition).name === "string" &&
    typeof (obj as PluginDefinition).setup === "function"
  );
}

export async function loadTSPlugins(pluginsDir: string): Promise<LoadedPlugin[]> {
  const loaded: LoadedPlugin[] = [];

  let files: string[];
  try {
    files = await readdir(pluginsDir);
  } catch {
    return [];
  }

  const tsFiles = files.filter((f) => f.endsWith(".ts") && !f.endsWith(".d.ts"));

  for (const file of tsFiles) {
    const filePath = resolve(join(pluginsDir, file));
    try {
      const mod = await import(filePath);
      const definition = mod.default ?? mod;

      if (!isPluginDefinition(definition)) {
        process.stderr.write(
          `[PluginLoader] "${file}" does not export a valid PluginDefinition. Skipping.\n`,
        );
        continue;
      }

      loaded.push({ name: definition.name, type: "typescript", definition });
      process.stderr.write(`[plugin] Loaded TS plugin: ${definition.name}\n`);
    } catch (err) {
      process.stderr.write(
        `[PluginLoader] Failed to load "${file}": ${err instanceof Error ? err.message : err}\n`,
      );
    }
  }

  return loaded;
}
