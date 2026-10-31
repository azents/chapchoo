import { readFile, readdir } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

// Staging is exercised in its real app directories; no independent asset layout.
test("both web distributions stage shared fonts, licenses and identity art", async () => {
  for (const app of ["web", "admin-web"]) {
    const result = spawnSync(process.execPath, [
      fileURLToPath(new URL("./stage-web-fonts.mjs", import.meta.url)),
      app,
    ]);
    assert.equal(result.status, 0, result.stderr.toString());
    const root = new URL(`../apps/${app}/public/`, import.meta.url);
    for (const font of ["GeistMono-Variable.woff2"]) {
      const bytes = await readFile(new URL(`fonts/${font}`, root));
      assert.equal(bytes.subarray(0, 4).toString(), "wOF2");
    }
    const stylesheet = await readFile(
      new URL("fonts/pretendard-subsets.css", root),
      "utf8",
    );
    const subsets = [...stylesheet.matchAll(/url\(\.\/pretendard\/([^)]+)\)/g)];
    assert.equal(subsets.length, 92);
    assert.equal((stylesheet.match(/unicode-range:/g) ?? []).length, 92);
    for (const match of subsets) {
      const bytes = await readFile(
        new URL(`fonts/pretendard/${match[1]}`, root),
      );
      assert.equal(bytes.subarray(0, 4).toString(), "wOF2");
      assert.ok(bytes.length < 45_000);
    }
    assert.ok(
      !(await readdir(new URL("fonts/", root))).includes(
        "PretendardVariable.woff2",
      ),
    );
    assert.match(
      await readFile(new URL("fonts/Pretendard-LICENSE.txt", root), "utf8"),
      /SIL OPEN FONT LICENSE/i,
    );
    assert.deepEqual(
      await readFile(new URL("design-assets/03-corgi-216.webp", root)),
      await readFile(
        new URL(
          "../packages/ui/assets/illustrations/03-corgi-216.webp",
          import.meta.url,
        ),
      ),
    );
    const published = await readdir(new URL("design-assets/", root));
    assert.deepEqual(published.sort(), [
      "01-retriever-216.webp",
      "02-bichon-216.webp",
      "03-corgi-216.webp",
      "04-dachshund-216.webp",
      "05-husky-216.webp",
      "06-border-collie-216.webp",
      "icons",
    ]);
    const characters = [
      "01-retriever",
      "02-bichon",
      "03-corgi",
      "04-dachshund",
      "05-husky",
      "06-border-collie",
    ];
    assert.deepEqual(
      (await readdir(new URL("design-assets/icons/", root))).sort(),
      characters,
    );
    for (const character of characters) {
      for (const [name, size] of [
        ["favicon-16.png", 16],
        ["favicon-32.png", 32],
        ["favicon-48.png", 48],
        ["apple-touch-icon.png", 180],
        ["icon-192.png", 192],
        ["icon-512.png", 512],
        ["icon-1024.png", 1024],
        ["icon-maskable-512.png", 512],
      ]) {
        const icon = await readFile(
          new URL(`design-assets/icons/${character}/${name}`, root),
        );
        assert.equal(icon.subarray(1, 4).toString(), "PNG");
        assert.equal(icon.readUInt32BE(16), size);
        assert.equal(icon.readUInt32BE(20), size);
        if (name === "apple-touch-icon.png" || name.startsWith("icon-")) {
          assert.equal(icon[25], 2, "home-screen artwork is opaque RGB");
        }
      }
      const ico = await readFile(
        new URL(`design-assets/icons/${character}/favicon.ico`, root),
      );
      assert.equal(ico.readUInt16LE(2), 1);
      assert.equal(ico.readUInt16LE(4), 3);
      assert.deepEqual([ico[6], ico[22], ico[38]], [16, 32, 48]);
    }
    assert.deepEqual(
      await readFile(new URL("favicon.ico", root)),
      await readFile(
        new URL("design-assets/icons/01-retriever/favicon.ico", root),
      ),
    );
  }
});
