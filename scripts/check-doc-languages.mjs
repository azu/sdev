import { readdir, readFile } from "node:fs/promises";
import { basename, join } from "node:path";

const files = new Set(["README.md", "README.ja.md", "AGENTS.md", "AGENTS.ja.md"]);
async function discover(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await discover(path);
    else if (entry.isFile() && path.endsWith(".md")) files.add(path);
  }
}
await discover("docs");
await discover("licenses");
let pairs = 0;
for (const path of files) {
  if (path.endsWith(".ja.md")) {
    if (!files.has(path.replace(/\.ja\.md$/, ".md"))) {
      throw new Error(`Missing English version for ${path}`);
    }
    continue;
  }
  const japanese = path.replace(/\.md$/, ".ja.md");
  const [english, translation] = await Promise.all([
    readFile(path, "utf8"),
    readFile(japanese, "utf8"),
  ]);
  if (!english.trim() || !translation.trim()) {
    throw new Error(`Empty language version: ${path}`);
  }
  // Pairing and navigation are mechanical checks; equivalent meaning still needs review.
  for (const [body, target] of [
    [english, basename(japanese)],
    [translation, basename(path)],
  ]) {
    if (!body.includes(`](${target})`)) {
      throw new Error(`Missing language navigation in ${path}: ${target}`);
    }
  }
  pairs += 1;
}
console.log(`Language pairs: ${pairs} English/Japanese documents`);
