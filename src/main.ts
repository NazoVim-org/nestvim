import { Editor } from "./editor";

async function main() {
  const filePath = process.argv[2];
  const editor = new Editor();
  await editor.run(filePath);
}

main().catch((err) => {
  process.stderr.write(`[editor] Fatal error: ${err instanceof Error ? err.message : err}\n`);
  process.exit(1);
});
