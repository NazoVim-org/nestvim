import { Editor } from "./editor";

async function main() {
  console.log("nazoedit starting...");
  const editor = new Editor();
  await editor.run();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});