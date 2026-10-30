import { copyFile, mkdir } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const name = process.argv[2];
if (name !== "chapchoo-web" && name !== "chapchoo-admin-web") {
  throw new Error("Select a Chapchoo web application");
}
const app = fileURLToPath(new URL(`../apps/${name}/`, import.meta.url));
const require = createRequire(join(app, "package.json"));
const distribution = dirname(require.resolve("geist/font/sans"));
const output = join(app, "public/fonts");
await mkdir(output, { recursive: true });
for (const [source, target] of [
  ["fonts/geist-sans/Geist-Variable.woff2", "Geist-Variable.woff2"],
  ["fonts/geist-mono/GeistMono-Variable.woff2", "GeistMono-Variable.woff2"],
  ["../LICENSE.txt", "LICENSE.txt"],
]) {
  await copyFile(join(distribution, source), join(output, target));
}
