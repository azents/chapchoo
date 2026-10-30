import { readFile } from "node:fs/promises";
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
    for (const font of [
      "PretendardVariable.woff2",
      "GeistMono-Variable.woff2",
    ]) {
      const bytes = await readFile(new URL(`fonts/${font}`, root));
      assert.equal(bytes.subarray(0, 4).toString(), "wOF2");
    }
    assert.match(
      await readFile(new URL("fonts/Pretendard-LICENSE.txt", root), "utf8"),
      /SIL OPEN FONT LICENSE/i,
    );
    assert.deepEqual(
      await readFile(new URL("design-assets/02-corgi.png", root)),
      await readFile(
        new URL("../packages/ui/assets/mascots/02-corgi.png", import.meta.url),
      ),
    );
  }
});
