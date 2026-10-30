import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";
import { FontFaces } from "./FontFaces";

await test("font preloads and CSS use the same runtime mount and font assets", () => {
  for (const basePath of ["", "/app", "/different/mount"]) {
    const html = renderToStaticMarkup(<FontFaces basePath={basePath} />);
    for (const name of ["PretendardVariable", "GeistMono-Variable"]) {
      const path = `${basePath}/fonts/${name}.woff2`;
      assert.ok(html.includes(`href="${path}"`));
      assert.ok(html.includes(`url("${path}")`));
    }
    assert.ok(html.includes('--font-chapchoo-sans: "Pretendard"'));
    assert.ok(html.includes("font-weight: 100 900"));
  }
});
