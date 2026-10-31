import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

async function choose(page: Page, label: string, value: string): Promise<void> {
  const trigger = page.getByRole("button", { name: "Appearance", exact: true });
  await trigger.click();
  await page.getByRole("menuitem", { name: label, exact: true }).click();
  await expect(trigger).toHaveAttribute("data-mode", value);
  await expect(trigger).toBeFocused();
}

test("icon menu follows live system appearance without replacing entered work", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto(
    "/iframe.html?id=common-ui-appearance--interactive&viewMode=story",
  );
  const mode = page.getByRole("button", { name: "Appearance", exact: true });
  const draft = page.getByRole("textbox", { name: "Draft" });
  await expect(mode).toBeVisible();
  await expect(mode).toHaveText("");
  await expect(mode).toHaveAttribute("title", "Appearance");
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "dark",
  );
  await draft.fill("보존할 draft with descenders gyp");
  await choose(page, "Light", "light");
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "light",
  );
  await page.emulateMedia({ colorScheme: "light" });
  await choose(page, "Dark", "dark");
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "dark",
  );
  await choose(page, "System", "system");
  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "dark",
  );
  await expect(draft).toHaveValue("보존할 draft with descenders gyp");
  await mode.press("Enter");
  const system = page.getByRole("menuitem", { name: "System", exact: true });
  await expect(system).toHaveAttribute("aria-current", "true");
  // Start keyboard navigation after Mantine's opening autofocus has settled.
  await expect(
    page.getByRole("menu").locator("[data-autofocus]"),
  ).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await expect(system).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await expect(
    page.getByRole("menuitem", { name: "Light", exact: true }),
  ).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(mode).toHaveAttribute("data-mode", "light");
  await expect(mode).toBeFocused();
  await page.setViewportSize({ width: 320, height: 800 });
  await mode.click();
  const menu = await page.getByRole("menu").boundingBox();
  expect(menu).not.toBeNull();
  if (menu) {
    expect(menu.x + menu.width).toBeLessThanOrEqual(320);
  }
  await page.keyboard.press("Escape");
  await expect(mode).toBeFocused();
  await mode.click();
  // The menu overlaps the input center; its exposed edge is an outside target.
  await draft.click({ position: { x: 8, y: 8 } });
  await expect(draft).toBeFocused();
  await expect(page.getByRole("menu")).toHaveCount(0);
});

test("auth and workspace share the top-right header position at every width", async ({
  page,
}) => {
  for (const width of [1280, 390, 320]) {
    await page.setViewportSize({ width, height: 950 });
    for (const id of [
      "accounts-authentication--password-login",
      "accounts-workspace--owner",
      "shared-ui-app-header--long-identity",
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
      await page.evaluate(() =>
        window.scrollTo(0, document.documentElement.scrollHeight),
      );
      const scrolled = await header.boundingBox();
      expect(scrolled?.y).toBe(0);
    }
  }
});
