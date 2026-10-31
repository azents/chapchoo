import { expect, test } from "@playwright/test";

test("long identity remains inside a growing header at 200% text size", async ({
  page,
}) => {
  for (const width of [769, 800]) {
    await page.setViewportSize({ width, height: 950 });
    for (const id of [
      "shared-ui-app-header--long-identity",
      "accounts-workspace--long-identity",
    ]) {
      await page.goto(
        `/iframe.html?id=${id}&viewMode=story&globals=colorScheme:dark`,
      );
      const header = page.getByRole("banner");
      const mode = header.getByRole("button", {
        name: "Appearance",
        exact: true,
      });
      await expect(mode).toBeVisible();
      await page.evaluate(() => {
        document.documentElement.style.fontSize = "200%";
      });
      const box = await header.boundingBox();
      const identity = await header
        .getByText(
          "long-account-name-with-an-unbroken-identifier-that-must-remain-readable@example.com",
          { exact: true },
        )
        .boundingBox();
      expect(box).not.toBeNull();
      expect(identity).not.toBeNull();
      if (box && identity) {
        expect(identity.y).toBeGreaterThanOrEqual(box.y);
        expect(identity.y + identity.height).toBeLessThanOrEqual(
          box.y + box.height,
        );
        if (id.startsWith("accounts-")) {
          const main = await page.getByRole("main").boundingBox();
          expect(main?.y).toBeGreaterThanOrEqual(box.y + box.height);
        }
      }
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth),
      ).toBeLessThanOrEqual(width);
      await mode.click();
      const menu = await page.getByRole("menu").boundingBox();
      expect(menu).not.toBeNull();
      if (menu) {
        expect(menu.x + menu.width).toBeLessThanOrEqual(width);
      }
      await page.keyboard.press("Escape");
      await expect(mode).toBeFocused();
    }
  }
});
