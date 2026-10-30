import { copyFile, mkdir, readdir, rm } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const name = process.argv[2];
if (name !== "web" && name !== "admin-web") {
  throw new Error("Select a Chapchoo web application");
}
const app = fileURLToPath(new URL(`../apps/${name}/`, import.meta.url));
const shared = fileURLToPath(
  new URL("../packages/ui/assets/", import.meta.url),
);
const output = join(app, "public/fonts");
await mkdir(output, { recursive: true });
await rm(join(output, "Geist-Variable.woff2"), { force: true });
await rm(join(output, "LICENSE.txt"), { force: true });
for (const asset of await readdir(join(shared, "fonts"))) {
  await copyFile(join(shared, "fonts", asset), join(output, asset));
}
const illustrations = join(app, "public/design-assets");
await mkdir(illustrations, { recursive: true });
for (const asset of await readdir(join(shared, "mascots"))) {
  await copyFile(join(shared, "mascots", asset), join(illustrations, asset));
}
