import { expect, test } from "@playwright/test";

test("Admin inline issuance fields and actions share the common frame height", async ({
  page,
}) => {
  for (const width of [1280, 390]) {
    await page.setViewportSize({ width, height: 850 });
    await page.goto(
      "/iframe.html?id=admin-grant-inventory--password-resets&viewMode=story",
    );
    const input = page.locator('main form input[type="email"]');
    const action = page.locator('main form button[type="submit"]');
    await expect(input).toBeVisible();
    const field = await input.boundingBox();
    const button = await action.boundingBox();
    expect(field?.height).toBe(44);
    expect(button?.height).toBe(field?.height);
    expect(button?.y).toBe(field?.y);
  }
});

test("Admin uses the independently scrolling shared desktop shell", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 300 });
  await page.goto(
    "/iframe.html?id=admin-grant-inventory--inventory&viewMode=story",
  );
  const footer = page.locator("aside img");
  const workspace = page.getByRole("main").locator("..");
  await expect(footer).toBeVisible();
  const initial = await footer.boundingBox();
  await workspace.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  expect(
    await workspace.evaluate((element) => element.scrollTop),
  ).toBeGreaterThan(0);
  expect(await footer.boundingBox()).toEqual(initial);
  expect(
    await page.evaluate(() => document.documentElement.scrollHeight),
  ).toBeLessThanOrEqual(300);
});
