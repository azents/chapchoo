import { expect, test } from "@playwright/test";

for (const width of [390, 1280]) {
  test(`Chat owns the scroll boundary and preserves header/composer at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.goto(
      "/iframe.html?id=chat-conversation--connected&viewMode=story",
    );
    const log = page.getByRole("log");
    const composer = page.getByRole("textbox", { name: "메시지", exact: true });
    const header = page.getByRole("main").locator("header");
    await expect(composer).toBeVisible();
    const before = {
      header: await header.boundingBox(),
      composer: await composer.boundingBox(),
    };
    await log.evaluate((element) => {
      element.scrollTop = 0;
    });
    await log.hover({ position: { x: 40, y: 320 } });
    await page.mouse.wheel(0, 500);
    await expect
      .poll(() => log.evaluate((element) => element.scrollTop))
      .toBeGreaterThan(0);
    await expect.poll(() => page.evaluate(() => window.scrollY)).toBe(0);
    expect(await header.boundingBox()).toEqual(before.header);
    expect(await composer.boundingBox()).toEqual(before.composer);
    await log.evaluate((element) => {
      element.scrollTop = element.scrollHeight;
    });
    await page.mouse.wheel(0, 1000);
    expect(
      await page.evaluate(() => ({
        y: scrollY,
        page: document.documentElement.scrollHeight,
        viewport: innerHeight,
      })),
    ).toEqual({ y: 0, page: 844, viewport: 844 });
  });
}

test("idle header is name-only and the detail sheet includes every record kind", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=chat-conversation--idle&viewMode=story");
  await expect(
    page.getByRole("button", { name: "코기 · 작업 상태", exact: true }),
  ).toHaveText("코기");
  await page.goto(
    "/iframe.html?id=chat-conversation--review-all-records&viewMode=story",
  );
  await page.getByRole("button", { name: /코기.*작업 상태/ }).click();
  const activity = page.getByRole("dialog", { name: "작업 상태", exact: true });
  await activity
    .getByRole("button", { name: "전체 기록", exact: true })
    .click();
  const dialog = page.getByRole("dialog", { name: "전체 기록", exact: true });
  for (const kind of [
    "user",
    "assistant",
    "thought",
    "plan",
    "tool",
    "terminal",
    "request",
    "state",
    "notice",
  ]) {
    await expect
      .poll(() => dialog.locator(`[data-kind="${kind}"]`).count())
      .toBeGreaterThan(0);
  }
  await dialog
    .getByRole("button", { name: /이 명령 실행을 허용할까요/ })
    .click();
  await expect(dialog.getByRole("radio").first()).toBeDisabled();
  expect(
    await page.evaluate(() => {
      const ids = [...document.querySelectorAll("[id]")].map(
        (element) => element.id,
      );
      return ids.filter((id, index) => ids.indexOf(id) !== index);
    }),
  ).toEqual([]);
});
