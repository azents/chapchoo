import { expect, test } from "@playwright/test";

test("context divider rows share the section baseline without a filled surface", async ({
  page,
}) => {
  for (const width of [1280, 390, 320]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(
      "/iframe.html?id=accounts-workspace--contexts-and-invitation&viewMode=story",
    );
    const row = page.locator('main [data-variant="row"]').first();
    await expect(row).toBeVisible();
    const link = await row.getByRole("link").boundingBox();
    const heading = await page
      .getByRole("main")
      .getByRole("heading", { level: 1 })
      .boundingBox();
    expect(link?.x).toBe(heading?.x);
    expect(
      await page.evaluate(
        () =>
          document.documentElement.scrollWidth <=
          document.documentElement.clientWidth,
      ),
    ).toBe(true);
  }
});
