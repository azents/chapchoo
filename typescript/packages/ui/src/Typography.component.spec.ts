import { expect, test } from "@playwright/test";

test("custom narrow editing CSS preserves the minimum and enlarged text", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto(
    "/iframe.html?id=shared-ui-fields--editable-controls&viewMode=story",
  );
  const editors = page.locator(
    'input:not([type="hidden"]),textarea,select,[contenteditable="true"],[contenteditable=""],[contenteditable="plaintext-only"]',
  );
  await expect(page.getByRole("searchbox", { name: "Search" })).toBeVisible();
  expect(await editors.count()).toBeGreaterThanOrEqual(13);
  for (const editor of await editors.all()) {
    await expect(editor).toHaveCSS("font-size", "16px");
    // Native selects retain their platform-owned line box.
    if (await editor.evaluate((element) => element.tagName !== "SELECT")) {
      await expect(editor).toHaveCSS("line-height", "20px");
    }
  }
  const search = page.getByRole("searchbox", { name: "Search" });
  const initialSize = await search.evaluate((element) =>
    parseFloat(getComputedStyle(element).fontSize),
  );
  await page.evaluate(() => {
    document.documentElement.style.fontSize = "200%";
  });
  expect(
    await search.evaluate((element) =>
      parseFloat(getComputedStyle(element).fontSize),
    ),
  ).toBeGreaterThan(initialSize);
});

test("shared action labels retain complete line boxes when wrapped or enlarged", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 950 });
  await page.goto(
    "/iframe.html?id=shared-ui-control-typography--labels&viewMode=story",
  );
  const labels = page.locator(".mantine-Button-label,.mantine-Chip-label");
  await expect(labels).toHaveCount(6);
  for (const size of ["100%", "200%"]) {
    await page.evaluate((value) => {
      document.documentElement.style.fontSize = value;
    }, size);
    for (const label of await labels.all()) {
      const metrics = await label.evaluate((element) => {
        const root = element.closest("button") ?? element;
        const box = root.getBoundingClientRect();
        const text = document.createRange();
        text.selectNodeContents(element);
        return {
          loading: root.hasAttribute("data-loading"),
          textFits: [...text.getClientRects()].every(
            (rect) => rect.top >= box.top && rect.bottom <= box.bottom,
          ),
        };
      });
      // Mantine intentionally translates the pending label behind its spinner.
      if (!metrics.loading) {
        expect(metrics.textFits).toBe(true);
      }
    }
  }
});
