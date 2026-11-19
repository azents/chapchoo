import { expect, test } from "@playwright/test";
import type { Locator } from "@playwright/test";

const story = (id: "initial" | "live"): string =>
  `/iframe.html?id=chat-work-summary--${id}&viewMode=story`;

function sizeFrames(
  element: Locator,
  times: [number, number, number],
): Promise<[number, number, number]> {
  return element.evaluate((surface, sampleTimes) => {
    const animation = surface.getAnimations()[0];
    if (!animation) {
      throw new Error("Work surface size animation is missing");
    }
    animation.pause();
    const heightAt = (time: number): number => {
      animation.currentTime = time;
      return surface.getBoundingClientRect().height;
    };
    const heights: [number, number, number] = [
      heightAt(sampleTimes[0]),
      heightAt(sampleTimes[1]),
      heightAt(sampleTimes[2]),
    ];
    animation.finish();
    return heights;
  }, times);
}

test("loaded work is settled and the keyboard opens the ordered tasks without moving the composer", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(story("initial"));
  const summary = page.getByRole("region", { name: "Current work" });
  const trigger = summary.getByRole("button");
  const composer = page.getByRole("textbox", { name: "Message" });
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
  expect(
    await summary.evaluate((element) => element.getAnimations().length),
  ).toBe(0);
  const anchor = await composer.boundingBox();
  await trigger.focus();
  await trigger.press("Enter");
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  await expect(summary.getByRole("listitem")).toHaveText([
    "Completed: Read the instructions",
    "In progress: Review source changes",
    "Pending: Summarize remaining work",
  ]);
  expect((await composer.boundingBox())?.y).toEqual(anchor?.y);
  await page.getByRole("button", { name: "Ask for input" }).click();
  await expect(trigger).toHaveAccessibleName(/Waiting for your reply/);
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
});

test("a live arrival grows and exit overshoots and retains content until removal", async ({
  page,
}) => {
  await page.goto(story("live"));
  const composer = page.getByRole("textbox", { name: "Message" });
  const anchor = await composer.boundingBox();
  const summary = page.locator('section[aria-label="Current work"]');
  await expect(summary).toHaveCount(0);
  await page.getByRole("button", { name: "Show work" }).click();
  const [start, middle, end] = await sizeFrames(summary, [0, 75, 579]);
  expect(start).toBeLessThan(middle);
  expect(middle).toBeLessThan(end);
  expect((await composer.boundingBox())?.y).toEqual(anchor?.y);
  await page.getByRole("button", { name: "Finish work" }).click();
  await expect(summary).toHaveAttribute("inert", "");
  await expect(summary.locator("button")).toHaveCSS("opacity", "1");
  const [exitStart, peak, nearlyGone] = await sizeFrames(summary, [0, 92, 400]);
  expect(peak).toBeGreaterThan(exitStart);
  expect(nearlyGone).toBeLessThan(exitStart);
  await expect(summary).toHaveCount(0);
  expect((await composer.boundingBox())?.y).toEqual(anchor?.y);
});

for (const theme of ["light", "dark"] as const) {
  test(`${theme} has an opaque title and a whole-surface press tint with reduced motion`, async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: theme, reducedMotion: "reduce" });
    await page.goto(`${story("initial")}&globals=colorScheme:${theme}`);
    const summary = page.getByRole("region", { name: "Current work" });
    const trigger = summary.getByRole("button");
    await expect(trigger).toBeVisible();
    const tint = (): Promise<string> =>
      summary.evaluate(
        (element) => getComputedStyle(element, "::before").opacity,
      );
    expect(await tint()).toBe("0");
    await trigger.dispatchEvent("pointerdown", { button: 0, isPrimary: true });
    expect(await tint()).toBe("0.08");
    await expect(trigger).toHaveCSS("opacity", "1");
    await trigger.dispatchEvent("pointercancel", { isPrimary: true });
    expect(await tint()).toBe("0");
    await trigger.click();
    expect(
      await summary.evaluate((element) => element.getAnimations().length),
    ).toBe(0);
    await page.getByRole("button", { name: "Finish work" }).click();
    await expect(summary).toHaveCount(0);
  });
}
