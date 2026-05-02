import { Editor } from "./editor";

async function main() {
  const filePath = process.argv[2];
  const editor = new Editor();
  await editor.run(filePath);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});