import { expect, test } from "@playwright/test";

const story = "/iframe.html?id=chat-presence--lifecycle&viewMode=story";

for (const width of [390, 1440]) {
  test(`composer error retains the composer width at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(
      "/iframe.html?id=chat-conversation--composer-request-error&viewMode=story",
    );
    const error = page.locator("footer [role=alert]");
    await expect(error).toBeVisible();
    await expect
      .poll(async () => {
        const errorBox = await error.boundingBox();
        const composer = await page
          .locator("footer [data-error]")
          .boundingBox();
        return errorBox && composer
          ? Math.abs(errorBox.width - composer.width)
          : null;
      })
      .toBeLessThan(1);
  });
}

test("initial content stays visible, new content starts collapsed and exits inert", async ({
  page,
}) => {
  await page.goto(story);
  const initial = page.getByTestId("initial");
  await expect(initial).toBeVisible();
  expect(
    await initial.evaluate(
      (node) => node.parentElement?.parentElement?.getAnimations().length,
    ),
  ).toBe(0);
  expect(
    await initial.evaluate((node) => {
      const outer = node.parentElement?.parentElement;
      return outer ? getComputedStyle(outer).overflow : null;
    }),
  ).toBe("visible");
  await page.getByRole("button", { name: "Toggle item" }).click();
  const item = page.getByTestId("dynamic");
  await expect(item).toBeAttached();
  const frames = await item.evaluate((node) => {
    const outer = node.parentElement?.parentElement;
    if (!outer) {
      throw new Error("Missing presence wrapper");
    }
    const animation = outer.getAnimations()[0];
    if (!animation) {
      throw new Error("Missing entry animation");
    }
    animation.pause();
    animation.currentTime = 0;
    const start = outer.getBoundingClientRect().height;
    animation.currentTime = 160;
    const middle = outer.getBoundingClientRect().height;
    animation.currentTime = 319;
    const end = outer.getBoundingClientRect().height;
    animation.play();
    return {
      start,
      middle,
      end,
      duration: animation.effect?.getTiming().duration,
    };
  });
  expect(frames.start).toBe(0);
  expect(frames.middle).toBeGreaterThan(0);
  expect(frames.middle).toBeLessThan(frames.end);
  expect(frames.duration).toBe(320);
  await expect
    .poll(() =>
      item.evaluate(
        (node) => node.parentElement?.parentElement?.getAnimations().length,
      ),
    )
    .toBe(0);
  await page.getByRole("button", { name: "Answer request" }).focus();
  expect(
    await item.evaluate((node) => {
      const outer = node.parentElement?.parentElement;
      return outer ? getComputedStyle(outer).overflow : null;
    }),
  ).toBe("visible");
  await page.getByRole("button", { name: "Toggle item" }).click();
  await expect(page.locator('[data-chat-presence="exiting"]')).toHaveCount(2);
  expect(await item.evaluate((node) => node.closest("[inert]") !== null)).toBe(
    true,
  );
  await expect(
    page.getByRole("button", { name: "Answer request" }),
  ).toHaveCount(0);
  await expect(item).toHaveCount(0);
});

test("reversal cancels late exit and prepended history does not animate", async ({
  page,
}) => {
  await page.goto(story);
  const toggle = page.getByRole("button", { name: "Toggle item" });
  await toggle.click();
  await expect(page.getByTestId("dynamic")).toBeVisible();
  await toggle.click();
  await expect(page.locator('[data-chat-presence="exiting"]')).toHaveCount(2);
  await toggle.click();
  await expect(
    page.getByRole("button", { name: "Answer request" }),
  ).toBeVisible();
  await expect
    .poll(() =>
      page
        .getByTestId("dynamic")
        .evaluate(
          (node) => node.parentElement?.parentElement?.getAnimations().length,
        ),
    )
    .toBe(0);
  await expect(page.getByTestId("dynamic")).toHaveCount(1);
  await page.getByRole("button", { name: "Prepend history" }).click();
  await expect(page.getByTestId("older")).toBeVisible();
  expect(
    await page
      .getByTestId("older")
      .evaluate(
        (node) => node.parentElement?.parentElement?.getAnimations().length,
      ),
  ).toBe(0);
  const previousFirstAnimated = await page
    .getByTestId("initial")
    .evaluate(async (node) => {
      const outer = node.parentElement?.parentElement;
      if (!outer) {
        throw new Error("Missing previous first-item wrapper");
      }
      // Sample one complete motion interval, rather than checking before ResizeObserver runs.
      const until = performance.now() + 320;
      let animated = false;
      while (performance.now() < until) {
        await new Promise<void>((resolve) =>
          requestAnimationFrame(() => resolve()),
        );
        animated ||= outer.getAnimations().length > 0;
      }
      return animated;
    });
  expect(previousFirstAnimated).toBe(false);
  await page.getByRole("button", { name: "Change source" }).click();
  await expect(page.getByTestId("dynamic")).toHaveCount(0);
  await expect(page.locator('[data-chat-presence="exiting"]')).toHaveCount(0);
});

test("reduced motion removes immediately and resizing settles at natural height", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(story);
  const toggle = page.getByRole("button", { name: "Toggle item" });
  await toggle.click();
  const item = page.getByTestId("dynamic");
  await expect(item).toBeVisible();
  expect(
    await item.evaluate(
      (node) => node.parentElement?.parentElement?.getAnimations().length,
    ),
  ).toBe(0);
  await page.getByRole("button", { name: "Resize item" }).click();
  await expect
    .poll(() =>
      item.evaluate((node) => {
        const inner = node.parentElement;
        return (
          inner?.parentElement?.getBoundingClientRect().height ===
          inner?.getBoundingClientRect().height
        );
      }),
    )
    .toBe(true);
  await toggle.click();
  await expect(item).toHaveCount(0);
});

test("custom-scrollbar reading remains detached throughout a new arrival", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--unread-messages&viewMode=story",
  );
  const log = page.getByRole("log", { name: "Message", exact: true });
  await expect
    .poll(() => log.evaluate((node) => node.scrollTop))
    .toBeGreaterThan(100);
  await log.hover();
  const thumb = page.locator(
    '.mantine-ScrollArea-scrollbar[data-orientation="vertical"] .mantine-ScrollArea-thumb',
  );
  await expect(thumb).toBeVisible();
  await expect
    .poll(() =>
      thumb.evaluate((node) => {
        const box = node.getBoundingClientRect();
        const track = node.parentElement?.getBoundingClientRect();
        return Boolean(
          track &&
          box.top >= track.top &&
          box.bottom <= track.bottom &&
          document.elementFromPoint(
            box.x + box.width / 2,
            box.y + box.height / 2,
          ) === node,
        );
      }),
    )
    .toBe(true);
  const bounds = await thumb.boundingBox();
  if (!bounds) {
    throw new Error("Missing scrollbar thumb geometry");
  }
  await page.mouse.move(
    bounds.x + bounds.width / 2,
    bounds.y + bounds.height / 2,
  );
  await page.mouse.down();
  await page.mouse.move(
    bounds.x + bounds.width / 2,
    bounds.y + bounds.height / 2 - 120,
    { steps: 8 },
  );
  await page.mouse.up();
  const before = await log.evaluate((node) => node.scrollTop);
  await expect
    .poll(() =>
      log.evaluate(
        (node) => node.scrollHeight - node.clientHeight - node.scrollTop,
      ),
    )
    .toBeGreaterThan(100);
  await page
    .getByRole("button", { name: "Receive message", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "New events below", exact: true }),
  ).toBeVisible();
  await expect
    .poll(() =>
      log.evaluate(
        (node) =>
          node
            .getAnimations({ subtree: true })
            .filter(
              (animation) => animation.effect?.getTiming().duration === 320,
            ).length,
      ),
    )
    .toBe(0);
  await expect
    .poll(() => log.evaluate((node) => node.scrollTop))
    .toBeCloseTo(before, 0);
});
