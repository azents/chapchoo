import assert from "node:assert/strict";
import test from "node:test";
import { readFile } from "node:fs/promises";
import { renderToStaticMarkup } from "react-dom/server";
import { FontFaces } from "./FontFaces";

await test("critical font preloading and lazy mono use the caller's runtime mount", () => {
  for (const basePath of ["", "/app", "/different/mount"]) {
    const html = renderToStaticMarkup(<FontFaces basePath={basePath} />);
    assert.ok(html.includes(`href="${basePath}/fonts/pretendard-subsets.css"`));
    assert.ok(html.includes('rel="stylesheet"'));
    assert.ok(
      html.includes(`url("${basePath}/fonts/GeistMono-Variable.woff2")`),
    );
    assert.ok(html.includes('rel="preload"'));
    assert.ok(
      html.includes(
        `href="${basePath}/fonts/pretendard/PretendardVariable.subset.91.woff2"`,
      ),
    );
    assert.ok(html.includes('as="font"'));
    assert.ok(html.includes('crossorigin="anonymous"'));
    assert.ok(html.includes("font-display: optional"));
    assert.ok(html.includes('--font-chapchoo-sans: "Pretendard"'));
    assert.ok(html.includes("font-weight: 100 900"));
  }
});

await test("all shipped text font subsets avoid late replacement after fallback paint", async () => {
  const css = await readFile(
    new URL("../assets/fonts/pretendard-subsets.css", import.meta.url),
    "utf8",
  );
  const faces = css.match(/@font-face\s*\{[^}]*\}/g) ?? [];
  assert.ok(faces.length > 0);
  for (const face of faces) {
    assert.match(face, /font-display:\s*optional;/);
  }
});
