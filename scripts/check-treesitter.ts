import { Language, Parser } from "web-tree-sitter";

await Parser.init();
const parser = new Parser();
const wasmPath = require.resolve("tree-sitter-typescript/tree-sitter-typescript.wasm");
console.log("Loading WASM from:", wasmPath);
const lang = await Language.load(wasmPath);
parser.setLanguage(lang);
const tree = parser.parse("const x: number = 42;");
if (!tree) {
  console.error("Failed to parse");
  process.exit(1);
}
console.log("Root node type:", tree.rootNode.type);
