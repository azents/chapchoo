import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";
import { FontFaces } from "./FontFaces";

await test("font stylesheet and lazy mono face use the caller's runtime mount", () => {
  for (const basePath of ["", "/app", "/different/mount"]) {
    const html = renderToStaticMarkup(<FontFaces basePath={basePath} />);
    assert.ok(html.includes(`href="${basePath}/fonts/pretendard-subsets.css"`));
    assert.ok(html.includes('rel="stylesheet"'));
    assert.ok(
      html.includes(`url("${basePath}/fonts/GeistMono-Variable.woff2")`),
    );
    assert.ok(!html.includes('rel="preload"'));
    assert.ok(html.includes('--font-chapchoo-sans: "Pretendard"'));
    assert.ok(html.includes("font-weight: 100 900"));
  }
});
