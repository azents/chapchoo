import { expect, test } from "@playwright/test";

const locales: [string, string][] = [
  ["korean", "생각 중"],
  ["english", "Thinking"],
  ["japanese", "考え中"],
  ["french", "Réflexion en cours"],
];
for (const [story, label] of locales) {
  test(`thinking is a short accessible status in ${story}`, async ({
    page,
  }) => {
    await page.goto(
      `/iframe.html?id=chat-thinking-indicator--${story}&viewMode=story`,
    );
    const status = page.getByRole("status");
    await expect(status).toContainText(label);
    await expect(status.locator('[aria-hidden="true"]')).toHaveText("✦");
    await expect
      .poll(() =>
        status.evaluate(
          (element) => element.getAnimations({ subtree: true }).length,
        ),
      )
      .toBe(2);
  });
}

test("reduced motion keeps the status readable without animation", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(
    "/iframe.html?id=chat-thinking-indicator--korean&viewMode=story",
  );
  const status = page.getByRole("status");
  await expect(status).toContainText("생각 중");
  expect(
    await status.evaluate(
      (element) => element.getAnimations({ subtree: true }).length,
    ),
  ).toBe(0);
  await expect(status.locator("span").last()).not.toHaveCSS(
    "color",
    "rgba(0, 0, 0, 0)",
  );
});

for (const width of [390, 1280]) {
  test(`working indicator follows the transcript and stays above the composer at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.goto(
      "/iframe.html?id=chat-conversation--connected&viewMode=story",
    );
    const thinking = page.getByRole("status").filter({ hasText: "생각 중" });
    await expect(thinking).toBeVisible();
    const bounds = await thinking.boundingBox();
    const composer = await page
      .getByRole("textbox", { name: "메시지", exact: true })
      .boundingBox();
    expect(bounds).not.toBeNull();
    expect(composer).not.toBeNull();
    if (!bounds || !composer) {
      throw new Error("Missing chat geometry");
    }
    expect(bounds.y + bounds.height).toBeLessThanOrEqual(composer.y);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
    ).toBeLessThanOrEqual(width);
  });
}
