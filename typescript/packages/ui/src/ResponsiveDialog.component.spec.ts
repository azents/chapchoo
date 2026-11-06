import { expect, test } from "@playwright/test";

for (const width of [320, 390]) {
  test(`shared bottom sheet fills ${width}px with bounded content height`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    const heights: number[] = [];
    for (const scene of ["short", "long"]) {
      await page.goto(
        `/iframe.html?id=shared-ui-responsive-dialog--${scene}&viewMode=story`,
      );
      const dialog = page.getByRole("dialog");
      await expect(dialog).toBeVisible();
      await expect
        .poll(async () => {
          const bounds = await dialog.boundingBox();
          return bounds ? bounds.y + bounds.height : Number.POSITIVE_INFINITY;
        })
        .toBeLessThanOrEqual(845);
      const box = await dialog.boundingBox();
      if (!box) {
        throw new Error("The shared sheet must have measurable bounds");
      }
      expect(box.x).toBeCloseTo(0, 0);
      expect(box.width).toBeCloseTo(width, 0);
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.height).toBeLessThanOrEqual(844 * 0.9 + 1);
      heights.push(box.height);
      await page
        .getByRole("button", { name: "Save", exact: true })
        .scrollIntoViewIfNeeded();
      await expect(
        page.getByRole("button", { name: "Save", exact: true }),
      ).toBeInViewport();
    }
    expect(heights[1]).toBeGreaterThan(heights[0] ?? 0);
  });
}

test("shared locked dialog prevents dismissal while desktop retains a modal", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=shared-ui-responsive-dialog--locked&viewMode=story",
  );
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Close", exact: true }),
  ).toHaveCount(0);
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(
    "/iframe.html?id=shared-ui-responsive-dialog--desktop&viewMode=story",
  );
  await expect(page.locator(".mantine-Modal-content")).toBeVisible();
  await expect(page.locator(".mantine-Drawer-content")).toHaveCount(0);
});
