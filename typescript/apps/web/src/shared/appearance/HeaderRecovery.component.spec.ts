import { expect, test } from "@playwright/test";

test("global appearance entry point remains mounted through workspace recovery", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=accounts-workspace--retry-to-loaded&viewMode=story",
  );
  await expect(page.getByRole("alert")).toBeVisible();
  const mode = page.getByRole("button", { name: "Appearance", exact: true });
  await expect(mode).toBeVisible();
  const original = await mode.elementHandle();
  expect(original).not.toBeNull();
  await page.getByRole("button", { name: "Try again", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Your spaces", exact: true }),
  ).toBeVisible();
  expect(await original.evaluate((element) => element.isConnected)).toBe(true);
  await expect(page.getByRole("banner")).toHaveCount(1);
  await mode.click();
  await expect(
    page.getByRole("menuitem", { name: "System", exact: true }),
  ).toHaveAttribute("aria-current", "true");
  await page.keyboard.press("Escape");
  await expect(mode).toBeFocused();
});
