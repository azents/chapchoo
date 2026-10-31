import assert from "node:assert/strict";
import test from "node:test";
import { MantineProvider } from "@mantine/core";
import { renderToStaticMarkup } from "react-dom/server";
import { Mascot } from "./Mascot";
import { mascotAssets } from "./designTokens";
import type { MascotId } from "./designTokens";

await test("decorative identity art respects root and nested runtime mounts", () => {
  for (const basePath of ["", "/app", "/another/mount"]) {
    const html = renderToStaticMarkup(
      <MantineProvider>
        <Mascot basePath={basePath} />
      </MantineProvider>,
    );
    assert.ok(
      html.includes(`src="${basePath}/design-assets/01-retriever-216.webp"`),
    );
    assert.ok(html.includes('alt=""'));
    assert.ok(html.includes('aria-hidden="true"'));
  }
});

await test("every character can be requested without changing the default", () => {
  const characters: MascotId[] = [
    "retriever",
    "bichon",
    "corgi",
    "dachshund",
    "husky",
    "collie",
  ];
  for (const character of characters) {
    const html = renderToStaticMarkup(
      <MantineProvider>
        <Mascot character={character} basePath="/nested/app" />
      </MantineProvider>,
    );
    assert.ok(
      html.includes(
        `/nested/app/design-assets/${mascotAssets[character].asset}-216.webp`,
      ),
    );
  }
});
