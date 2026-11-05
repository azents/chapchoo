import { expect, test } from "@playwright/test";
import messages from "../../../messages/en-US.json" with { type: "json" };

test("auth mascot keeps its layout and ignores replay input during desktop/touch playback", async ({
  browser,
  baseURL,
}) => {
  for (const touch of [false, true]) {
    const context = await browser.newContext({
      baseURL,
      viewport: { width: touch ? 390 : 1280, height: 900 },
      hasTouch: touch,
      reducedMotion: "reduce",
    });
    const page = await context.newPage();
    await page.goto(
      "/iframe.html?id=accounts-authentication--server-selected-character&viewMode=story",
    );
    const button = page.getByRole("button", {
      name: messages.accounts.replayMascot,
    });
    const motion = button.locator("[data-playing]");
    await expect(button).toBeDisabled();
    await expect(button).toHaveCSS("width", "72px");
    await expect(button).toHaveCSS("height", "72px");
    await expect(button).toHaveAttribute("type", "button");
    await expect(
      button.locator('img[src$="/05-husky-216.webp"]'),
    ).toBeVisible();
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await expect(motion).toHaveAttribute("data-playing", "true");
    await motion.evaluate((node) => {
      for (const animation of node.getAnimations()) {
        animation.pause();
        animation.id = "current-gesture";
      }
    });
    if (touch) {
      await button.tap();
      await button.tap();
    } else {
      await button.click();
      await button.click();
    }
    await button.press("Enter");
    expect(
      await motion.evaluate((node) =>
        node.getAnimations().map((animation) => animation.id),
      ),
    ).toEqual(["current-gesture"]);
    await motion.evaluate((node) => {
      for (const animation of node.getAnimations()) {
        animation.finish();
      }
    });
    await expect(motion).toHaveAttribute("data-playing", "false");
    await button.hover();
    await expect(motion).toHaveAttribute("data-playing", "false");
    if (touch) {
      await button.tap();
    } else {
      await button.press("Space");
    }
    await expect(motion).toHaveAttribute("data-playing", "true");
    await page.emulateMedia({ reducedMotion: "reduce" });
    await expect(button).toBeDisabled();
    await expect(motion).toHaveAttribute("data-playing", "false");
    await context.close();
  }
});
