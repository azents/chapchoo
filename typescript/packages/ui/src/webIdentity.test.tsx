import assert from "node:assert/strict";
import test from "node:test";
import { identityManifest, identityMetadata } from "./webIdentity";

await test("browser identity is rooted at the runtime mount for both apps", () => {
  for (const basePath of ["", "/app", "/nested/admin"]) {
    const metadata = identityMetadata(basePath);
    assert.equal(metadata.manifest, `${basePath}/manifest.webmanifest`);
    assert.equal(metadata.icons.shortcut, `${basePath}/favicon.ico`);
    for (const icon of [...metadata.icons.icon, ...metadata.icons.apple]) {
      assert.ok(
        icon.url.startsWith(`${basePath}/design-assets/icons/06-retriever/`),
      );
    }
    assert.deepEqual(
      metadata.icons.icon.map((icon) => icon.sizes),
      ["16x16", "32x32", "48x48"],
    );
    assert.equal(metadata.icons.apple[0]?.sizes, "180x180");
    for (const name of ["Chapchoo", "Chapchoo Admin"]) {
      const manifest = identityManifest(basePath, name);
      assert.equal(manifest.name, name);
      assert.equal(manifest.id, `${basePath}/`);
      assert.equal(manifest.scope, `${basePath}/`);
      assert.equal(manifest.start_url, `${basePath}/`);
      assert.equal(manifest.display, "browser");
      assert.deepEqual(
        manifest.icons.map((icon) => [icon.sizes, icon.purpose]),
        [
          ["192x192", "any"],
          ["512x512", "any"],
          ["512x512", "maskable"],
        ],
      );
      for (const icon of manifest.icons) {
        assert.ok(
          icon.src.startsWith(`${basePath}/design-assets/icons/06-retriever/`),
        );
      }
    }
  }
});

await test("a distinct character's prepared home-screen assets can be selected explicitly", () => {
  const manifest = identityManifest("/app", "Chapchoo", "husky");
  assert.ok(manifest.icons.every((icon) => icon.src.includes("/04-husky/")));
});
