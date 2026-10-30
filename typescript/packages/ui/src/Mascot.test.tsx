import assert from "node:assert/strict";
import test from "node:test";
import { MantineProvider } from "@mantine/core";
import { renderToStaticMarkup } from "react-dom/server";
import { Mascot } from "./Mascot";

await test("decorative identity art respects root and nested runtime mounts", () => {
  for (const basePath of ["", "/app", "/another/mount"]) {
    const html = renderToStaticMarkup(
      <MantineProvider>
        <Mascot basePath={basePath} />
      </MantineProvider>,
    );
    assert.ok(html.includes(`src="${basePath}/design-assets/02-corgi.png"`));
    assert.ok(html.includes('alt=""'));
    assert.ok(html.includes('aria-hidden="true"'));
  }
});
