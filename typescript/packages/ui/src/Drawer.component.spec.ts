import { expect, test } from "@playwright/test";

for (const width of [1280, 402, 320]) {
  test(`shared Drawer theme separates its first body content from the header at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    for (const scene of ["content", "wrapped-heading"]) {
      await page.goto(
        `/iframe.html?id=shared-ui-drawer--${scene}&viewMode=story`,
      );
      const opening = page.getByText("Body opening", { exact: true });
      await expect(opening).toBeVisible();
      await page.evaluate(() => document.fonts.ready);
      const body = await page.locator(".mantine-Drawer-body").boundingBox();
      const content = await opening.boundingBox();
      if (!body || !content) {
        throw new Error(
          "The shared Drawer fixture must have measurable content",
        );
      }
      // This theme owns the body inset, not Mantine's token-to-pixel resolution.
      expect(content.y).toBeGreaterThan(body.y);
    }
  });
}
