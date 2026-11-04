import { expect, test } from "@playwright/test";

test.use({ deviceScaleFactor: 1, colorScheme: "light" });
test("saved profile cards use two desktop columns, one mobile column and contained long names", async ({
  page,
}, info) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto(
    "/iframe.html?id=computers-agents--profile-cards&viewMode=story",
  );
  const first = page.getByRole("link", {
    name: "Research companion",
    exact: true,
  });
  const second = page.getByRole("link", {
    name: "A long-running research companion with a descriptive display name",
    exact: true,
  });
  await expect(first).toBeVisible();
  await expect(second).toBeVisible();
  await page.evaluate(async () => document.fonts.ready);
  const desktop = await first.boundingBox();
  const sibling = await second.boundingBox();
  if (!desktop || !sibling) {
    throw new Error("Profile cards have no desktop geometry");
  }
  expect(desktop.y).toBe(sibling.y);
  expect(sibling.x).toBeGreaterThan(desktop.x);
  const font = await first.evaluate(
    (element) => getComputedStyle(element).fontFamily,
  );
  expect(font).toContain("Pretendard");
  await page.screenshot({
    path: info.outputPath("saved-profile-cards-desktop.png"),
    fullPage: true,
  });
  await page.setViewportSize({ width: 375, height: 812 });
  const mobile = await first.boundingBox();
  const next = await second.boundingBox();
  if (!mobile || !next) {
    throw new Error("Profile cards have no mobile geometry");
  }
  expect(next.x).toBe(mobile.x);
  expect(next.y).toBeGreaterThan(mobile.y);
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: info.outputPath("saved-profile-cards-mobile.png"),
    fullPage: true,
  });
});
