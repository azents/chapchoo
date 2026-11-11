import { expect, test } from "@playwright/test";

test.use({
  viewport: { width: 402, height: 780 },
  isMobile: true,
  hasTouch: true,
});

test("a touch swipe over a recent record pans without activating it; a tap still opens it", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--review-all-records&viewMode=story",
  );
  await page.getByRole("button", { name: /코기.*작업 상태/ }).click();
  const sheet = page.getByRole("dialog", { name: "작업 상태", exact: true });
  const viewport = sheet.locator('[class*="panelBody"]');
  const row = sheet.locator('button[class*="recent"]').first();
  await expect
    .poll(() =>
      sheet.evaluate((element) =>
        Math.abs(element.getBoundingClientRect().bottom - innerHeight),
      ),
    )
    .toBeLessThan(0.1);
  await viewport.evaluate((element) => {
    element.scrollTop = 0;
  });
  const bounds = await row.boundingBox();
  if (!bounds) {
    throw new Error("Recent record is absent");
  }
  const before = await viewport.evaluate((element) => element.scrollTop);
  const client = await page.context().newCDPSession(page);
  const x = bounds.x + bounds.width / 2;
  const y = bounds.y + bounds.height / 2;
  await client.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [{ x, y }],
  });
  for (let step = 1; step <= 18; step++) {
    await client.send("Input.dispatchTouchEvent", {
      type: "touchMove",
      touchPoints: [{ x, y: y - step * 7 }],
    });
    await page.evaluate(() => new Promise(requestAnimationFrame));
  }
  await client.send("Input.dispatchTouchEvent", {
    type: "touchEnd",
    touchPoints: [],
  });
  await expect
    .poll(() => viewport.evaluate((element) => element.scrollTop))
    .toBeGreaterThan(before + 40);
  await expect(sheet).toBeVisible();
  await expect(
    page.getByRole("dialog", { name: "전체 기록", exact: true }),
  ).toHaveCount(0);
  await row.tap();
  await expect(
    page.getByRole("dialog", { name: "전체 기록", exact: true }),
  ).toBeVisible();
  await client.detach();
});

for (const surface of ["activity", "history"] as const) {
  test(`${surface} leaves outward wheel and touch gestures uncanceled at both body edges`, async ({
    page,
  }) => {
    await page.goto(
      "/iframe.html?id=chat-conversation--activity-sheet&viewMode=story",
    );
    if (surface === "history") {
      await page
        .getByRole("button", { name: "전체 기록", exact: true })
        .click();
    }
    const sheet = page.getByRole("dialog", {
      name: surface === "history" ? "전체 기록" : "작업 상태",
      exact: true,
    });
    await sheet.getByRole("button", { name: "닫기", exact: true }).hover();
    const viewport = sheet.locator("[data-sheet-viewport]");
    for (const edge of ["top", "bottom"] as const) {
      const canceled = await viewport.evaluate((element, boundary) => {
        element.scrollTop = boundary === "top" ? 0 : element.scrollHeight;
        const direction = boundary === "top" ? -1 : 1;
        const bounds = element.getBoundingClientRect();
        const x = bounds.x + bounds.width / 2;
        const y = bounds.y + bounds.height / 2;
        const start = new Touch({
          identifier: 1,
          target: element,
          clientX: x,
          clientY: y,
        });
        const moved = new Touch({
          identifier: 1,
          target: element,
          clientX: x,
          clientY: y - direction * 40,
        });
        const wheel = !element.dispatchEvent(
          new WheelEvent("wheel", {
            deltaY: direction * 120,
            bubbles: true,
            cancelable: true,
          }),
        );
        element.dispatchEvent(
          new TouchEvent("touchstart", {
            touches: [start],
            changedTouches: [start],
            targetTouches: [start],
            bubbles: true,
            cancelable: true,
          }),
        );
        const touch = !element.dispatchEvent(
          new TouchEvent("touchmove", {
            touches: [moved],
            changedTouches: [moved],
            targetTouches: [moved],
            bubbles: true,
            cancelable: true,
          }),
        );
        element.dispatchEvent(
          new TouchEvent("touchend", {
            touches: [],
            changedTouches: [moved],
            targetTouches: [],
            bubbles: true,
            cancelable: true,
          }),
        );
        return { wheel, touch };
      }, edge);
      expect(canceled, edge).toEqual({ wheel: false, touch: false });
    }
  });
}
