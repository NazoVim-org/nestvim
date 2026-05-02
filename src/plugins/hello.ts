import type { PluginDefinition } from "../plugin/types";

const plugin: PluginDefinition = {
  name: "hello-ts",
  version: "0.1.0",
  setup(api) {
    api.addCommand("hello", () => {
      api.log("Hello from TypeScript plugin!");
    });

    api.on("editor:ready", () => {
      api.log("hello-ts plugin ready");
    });
  },
};

export default plugin;