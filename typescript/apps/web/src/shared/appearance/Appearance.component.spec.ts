import { expect, test } from "@playwright/test";

test("three choices follow live system appearance without replacing entered work", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto(
    "/iframe.html?id=common-ui-appearance--interactive&viewMode=story",
  );
  const mode = page.getByRole("combobox", { name: "Appearance" });
  const draft = page.getByRole("textbox", { name: "Draft" });
  await expect(mode).toBeVisible();
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "dark",
  );
  await draft.fill("보존할 draft with descenders gyp");
  await mode.selectOption("light");
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "light",
  );
  await page.emulateMedia({ colorScheme: "light" });
  await mode.selectOption("dark");
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "dark",
  );
  await mode.selectOption("system");
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "light",
  );
  await page.emulateMedia({ colorScheme: "dark" });
  await expect(page.locator("html")).toHaveAttribute(
    "data-mantine-color-scheme",
    "dark",
  );
  await expect(draft).toHaveValue("보존할 draft with descenders gyp");
  await mode.focus();
  await page.keyboard.press("ArrowUp");
  await expect(mode).toBeFocused();
  await page.setViewportSize({ width: 320, height: 800 });
  const dimensions = await page.evaluate(() => ({
    width: innerWidth,
    scroll: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scroll).toBeLessThanOrEqual(dimensions.width);
});
