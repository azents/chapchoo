import { copyFile, cp, mkdir, readdir, rm } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  defaultMascot,
  mascotAssets,
} from "../packages/ui/src/designTokens.ts";

const name = process.argv[2];
if (name !== "web" && name !== "admin-web") {
  throw new Error("Select a Chapchoo web application");
}
const app = fileURLToPath(new URL(`../apps/${name}/`, import.meta.url));
const shared = fileURLToPath(
  new URL("../packages/ui/assets/", import.meta.url),
);
const output = join(app, "public/fonts");
await rm(output, { recursive: true, force: true });
await cp(join(shared, "fonts"), output, { recursive: true });
const illustrations = join(app, "public/design-assets");
await rm(illustrations, { recursive: true, force: true });
await mkdir(illustrations, { recursive: true });
for (const asset of await readdir(join(shared, "illustrations"))) {
  if (asset.endsWith(".webp")) {
    await copyFile(
      join(shared, "illustrations", asset),
      join(illustrations, asset),
    );
  }
}
const icons = join(illustrations, "icons");
await mkdir(icons, { recursive: true });
for (const entry of await readdir(join(shared, "icons"), {
  withFileTypes: true,
})) {
  if (entry.isDirectory()) {
    await cp(join(shared, "icons", entry.name), join(icons, entry.name), {
      recursive: true,
    });
  }
}
await copyFile(
  join(shared, "icons", mascotAssets[defaultMascot].asset, "favicon.ico"),
  join(app, "public/favicon.ico"),
);
