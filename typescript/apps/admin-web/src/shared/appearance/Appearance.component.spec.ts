import { expect, test } from "@playwright/test";

test("Admin auth, setup and inventory use the same global icon position", async ({
  page,
}) => {
  for (const width of [1280, 390, 320]) {
    await page.setViewportSize({ width, height: 950 });
    for (const id of [
      "admin-authentication--password-login",
      "admin-authentication--initial-setup",
      "admin-grant-inventory--inventory",
    ]) {
      await page.goto(
        `/iframe.html?id=${id}&viewMode=story&globals=colorScheme:dark`,
      );
      const mode = page
        .getByRole("banner")
        .getByRole("button", { name: "Appearance", exact: true });
      await expect(mode).toBeVisible();
      const target = await mode.boundingBox();
      expect(target).not.toBeNull();
      if (target) {
        expect(target.width).toBe(44);
        expect(target.height).toBe(44);
        expect(width - target.x - target.width).toBe(width > 768 ? 24 : 16);
      }
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth),
      ).toBeLessThanOrEqual(width);
      await mode.click();
      await expect(
        page.getByRole("menuitem", { name: "System", exact: true }),
      ).toHaveAttribute("aria-current", "true");
      await page.keyboard.press("Escape");
      await expect(mode).toBeFocused();
    }
  }
});
