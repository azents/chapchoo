import { expect, test } from "@playwright/test";

test("activity sheet keeps its heading and close action outside the scrolling content", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 600 });
  await page.goto(
    "/iframe.html?id=chat-conversation--review-all-records&viewMode=story",
  );
  const originalBodyOverflow = await page
    .locator("body")
    .evaluate((element) => getComputedStyle(element).overflow);
  await page.getByRole("button", { name: /코기.*작업 상태/ }).click();
  const dialog = page.getByRole("dialog", { name: "작업 상태", exact: true });
  const heading = dialog.getByRole("heading", {
    name: "작업 상태",
    exact: true,
  });
  const close = dialog.getByRole("button", { name: "닫기", exact: true });
  const region = dialog.locator('[class*="panelBody"]');
  await expect(heading).toBeVisible();
  await close.hover();
  await expect(page.locator("body")).toHaveCSS("overflow", "hidden");
  await expect(dialog).toHaveCSS("overflow", "clip");
  await expect(region).toHaveCSS("overflow-y", "auto");
  await expect(region).toHaveCSS("overscroll-behavior-y", "auto");
  const bar = await dialog.locator("header").boundingBox();
  const body = await region.boundingBox();
  expect(body?.y).toBeCloseTo((bar?.y ?? 0) + (bar?.height ?? 0), 0);
  const detail = dialog.getByRole("link", {
    name: "코기 에이전트 상세",
    exact: true,
  });
  await expect(detail).toContainText("코기");
  await expect(detail).toContainText("에이전트 상세");
  await expect(detail).toHaveAttribute("href", "/teams/my/agents/corgi");
  expect((await detail.boundingBox())?.y).toBeGreaterThanOrEqual(
    (bar?.y ?? 0) + (bar?.height ?? 0),
  );
  const before = {
    heading: await heading.boundingBox(),
    close: await close.boundingBox(),
  };
  await region.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  await expect
    .poll(() => region.evaluate((element) => element.scrollTop))
    .toBeGreaterThan(0);
  expect(await heading.boundingBox()).toEqual(before.heading);
  expect(await close.boundingBox()).toEqual(before.close);
  expect(await dialog.evaluate((element) => element.scrollTop)).toBe(0);
  await expect(close).toBeVisible();
  await close.click();
  await expect(dialog).toBeHidden();
  await expect(page.locator("body")).toHaveCSS(
    "overflow",
    originalBodyOverflow,
  );
});

test("a short conversation still has scroll travel", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/iframe.html?id=chat-conversation--idle&viewMode=story");
  const log = page.getByRole("log");
  const composer = page.getByRole("textbox", { name: "메시지", exact: true });
  const header = page.getByRole("main").locator("header");
  await expect(log).toBeVisible();
  await expect
    .poll(() =>
      log.evaluate((element) => element.scrollHeight - element.clientHeight),
    )
    .toBeGreaterThan(0);
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.clientHeight - element.scrollTop,
      ),
    )
    .toBeLessThanOrEqual(1);
  const before = {
    header: await header.boundingBox(),
    composer: await composer.boundingBox(),
    travel: await log.evaluate(
      (element) => element.scrollHeight - element.clientHeight,
    ),
  };
  await log.hover({ position: { x: 40, y: 320 } });
  await page.mouse.wheel(0, -before.travel);
  await expect
    .poll(() => log.evaluate((element) => element.scrollTop))
    .toBeLessThan(before.travel);
  await expect.poll(() => page.evaluate(() => window.scrollY)).toBe(0);
  expect(await header.boundingBox()).toEqual(before.header);
  expect(await composer.boundingBox()).toEqual(before.composer);
});

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
