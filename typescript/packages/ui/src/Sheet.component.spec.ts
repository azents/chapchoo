import { expect, test } from "@playwright/test";

for (const width of [390, 1280]) {
  test(`Sheet owns a native body boundary below stable chrome at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 700 });
    await page.goto(
      "/iframe.html?id=shared-sheet--full-history&viewMode=story",
    );
    const sheet = page.getByRole("dialog", { name: "History", exact: true });
    const viewport = sheet.getByRole("log", { name: "Records" });
    const header = sheet.locator("header");
    const close = sheet.getByRole("button", { name: "Close sheet" });
    await expect(close).toBeVisible();
    await expect
      .poll(() =>
        sheet.evaluate((e) =>
          Math.abs(e.getBoundingClientRect().bottom - innerHeight),
        ),
      )
      .toBeLessThan(0.1);
    const before = {
      header: await header.boundingBox(),
      close: await close.boundingBox(),
    };
    const body = await viewport.boundingBox();
    expect(body?.x).toBe(0);
    expect(body?.width).toBe(width);
    expect(
      (before.close?.x ?? 0) + (before.close?.width ?? 0),
    ).toBeLessThanOrEqual(width);
    expect(body?.y).toBeCloseTo(
      (before.header?.y ?? 0) + (before.header?.height ?? 0),
      0,
    );
    await viewport.evaluate((e) => {
      e.scrollTop = 100;
    });
    await expect
      .poll(() => viewport.evaluate((e) => e.scrollTop))
      .toBeGreaterThan(0);
    expect(await header.boundingBox()).toEqual(before.header);
    expect(await close.boundingBox()).toEqual(before.close);
    expect(await sheet.evaluate((e) => e.scrollTop)).toBe(0);
    await expect(viewport).toHaveCSS("mask-image", "none");
    await expect(page.locator("body")).toHaveCSS("overflow", "hidden");
    for (const edge of ["top", "bottom"] as const) {
      const canceled = await viewport.evaluate((e, edge) => {
        e.scrollTop = edge === "top" ? 0 : e.scrollHeight;
        return !e.dispatchEvent(
          new WheelEvent("wheel", {
            deltaY: edge === "top" ? -100 : 100,
            bubbles: true,
            cancelable: true,
          }),
        );
      }, edge);
      expect(canceled).toBe(false);
    }
    await close.click();
    await expect(sheet).toBeHidden();
    await expect
      .poll(() =>
        page.locator("body").evaluate((e) => getComputedStyle(e).overflow),
      )
      .not.toBe("hidden");
  });
}

test("fade is opt-in and nested modal returns focus to the same sheet", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=shared-sheet--activity-fade&viewMode=story");
  const sheet = page.getByRole("dialog", { name: "History", exact: true });
  const viewport = sheet.getByRole("log", { name: "Records" });
  await expect(viewport).not.toHaveCSS("mask-image", "none");
  const opener = sheet.getByRole("button", { name: "Open confirmation" });
  await opener.click();
  const modal = page.getByRole("dialog", { name: "Confirmation", exact: true });
  await expect(modal).toBeVisible();
  await modal.getByRole("button", { name: "Back to sheet" }).click();
  await expect(modal).toBeHidden();
  await expect(opener).toBeFocused();
  await expect(sheet).toBeVisible();
});
