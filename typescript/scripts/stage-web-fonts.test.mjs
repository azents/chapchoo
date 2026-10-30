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
      await readFile(new URL("design-assets/02-corgi-216.webp", root)),
      await readFile(
        new URL(
          "../packages/ui/assets/illustrations/02-corgi-216.webp",
          import.meta.url,
        ),
      ),
    );
    const published = await readdir(new URL("design-assets/", root));
    assert.deepEqual(published, ["02-corgi-216.webp"]);
  }
});
