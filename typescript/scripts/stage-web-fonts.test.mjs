import { cp, mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

async function equalTree(source, output) {
  const entries = await readdir(source, { withFileTypes: true });
  assert.deepEqual(
    (await readdir(output)).sort(),
    entries.map((entry) => entry.name).sort(),
  );
  for (const entry of entries) {
    const sourceFile = new URL(
      entry.name + (entry.isDirectory() ? "/" : ""),
      source,
    );
    const outputFile = new URL(
      entry.name + (entry.isDirectory() ? "/" : ""),
      output,
    );
    if (entry.isDirectory()) await equalTree(sourceFile, outputFile);
    else
      assert.deepEqual(await readFile(outputFile), await readFile(sourceFile));
  }
}

test("both web distributions preserve selected asset bytes and publish the default favicon", async () => {
  const assets = new URL("../packages/ui/assets/", import.meta.url);
  for (const app of ["web", "admin-web"]) {
    const result = spawnSync(process.execPath, [
      fileURLToPath(new URL("./stage-web-fonts.mjs", import.meta.url)),
      app,
    ]);
    assert.equal(result.status, 0, result.stderr.toString());
    const root = new URL(`../apps/${app}/public/`, import.meta.url);
    await equalTree(new URL("fonts/", assets), new URL("fonts/", root));
    const illustrations = (
      await readdir(new URL("illustrations/", assets))
    ).filter((name) => name.endsWith(".webp"));
    assert.deepEqual(
      (await readdir(new URL("design-assets/", root))).sort(),
      [...illustrations, "icons"].sort(),
    );
    for (const name of illustrations) {
      assert.deepEqual(
        await readFile(new URL(`design-assets/${name}`, root)),
        await readFile(new URL(`illustrations/${name}`, assets)),
      );
    }
    const characters = (
      await readdir(new URL("icons/", assets), { withFileTypes: true })
    )
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name);
    assert.deepEqual(
      (await readdir(new URL("design-assets/icons/", root))).sort(),
      characters.sort(),
    );
    for (const character of characters) {
      await equalTree(
        new URL(`icons/${character}/`, assets),
        new URL(`design-assets/icons/${character}/`, root),
      );
    }
    assert.deepEqual(
      await readFile(new URL("favicon.ico", root)),
      await readFile(new URL("icons/01-retriever/favicon.ico", assets)),
    );
    const configRoot = new URL(`../apps/${app}/.storybook/`, import.meta.url);
    const { default: config } = await import(new URL("main.ts", configRoot));
    const mappings = config.staticDirs;
    for (const left of mappings) {
      for (const right of mappings) {
        if (left === right) continue;
        assert.ok(
          !right.to.startsWith(`${left.to}/`) && right.to !== left.to,
          "Concurrent Storybook static copies must have disjoint destination trees",
        );
      }
    }
    const temporary = await mkdtemp(
      join(tmpdir(), "chapchoo-storybook-assets-"),
    );
    try {
      await Promise.all(
        mappings.map(({ from, to }) =>
          cp(resolve(fileURLToPath(configRoot), from), join(temporary, to), {
            recursive: true,
          }),
        ),
      );
      const copied = pathToFileURL(`${temporary}/`);
      await equalTree(
        new URL("design-assets/", root),
        new URL("design-assets/", copied),
      );
      await equalTree(new URL("fonts/", root), new URL("fonts/", copied));
    } finally {
      await rm(temporary, { recursive: true, force: true });
    }
  }
});
