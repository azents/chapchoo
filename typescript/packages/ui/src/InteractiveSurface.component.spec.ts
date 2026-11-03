import { expect, test } from "@playwright/test";
import type { Locator } from "@playwright/test";

async function background(target: Locator): Promise<string> {
  return target.evaluate(
    (element) => getComputedStyle(element).backgroundColor,
  );
}
for (const theme of ["light", "dark"]) {
  test(`interactive surfaces distinguish pointer and keyboard states in ${theme}`, async ({
    page,
  }) => {
    await page.goto(
      `/iframe.html?id=chrome-interactive-surface--states&viewMode=story&globals=colorScheme:${theme}`,
    );
    const action = page.getByRole("button", {
      name: "Open settings",
      exact: true,
    });
    const initial = await background(action);
    await action.hover();
    await expect.poll(() => background(action)).not.toBe(initial);
    const hover = await background(action);
    await page.mouse.down();
    await expect.poll(() => background(action)).not.toBe(hover);
    await expect(page.getByRole("status")).toHaveText("Actions: 0");
    await page.mouse.move(1, 1);
    await page.mouse.up();
    await expect(page.getByRole("status")).toHaveText("Actions: 0");
    await action.click();
    await expect(page.getByRole("status")).toHaveText("Actions: 1");
    const disabled = page.getByRole("button", {
      name: "Unavailable action",
      exact: true,
    });
    const disabledBackground = await background(disabled);
    await disabled.hover();
    expect(await background(disabled)).toBe(disabledBackground);
    await page.reload();
    await action.focus();
    await page.keyboard.press("Shift+Tab");
    const link = page.getByRole("link", { name: "Open profile", exact: true });
    await expect(link).toBeFocused();
    await expect(link).toHaveCSS("outline-style", "solid");
    await expect(link).toHaveCSS("outline-width", "2px");
    const size = await link.boundingBox();
    expect(size?.height).toBeGreaterThanOrEqual(44);
    await page.keyboard.press("Enter");
    await expect(page).toHaveURL(/#profile$/);
  });
}

test("surface feedback respects reduced motion and forced colors", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce", forcedColors: "active" });
  await page.goto(
    "/iframe.html?id=chrome-interactive-surface--states&viewMode=story",
  );
  const action = page.getByRole("button", {
    name: "Open settings",
    exact: true,
  });
  await action.focus();
  await page.keyboard.press("Shift+Tab");
  const link = page.getByRole("link", { name: "Open profile", exact: true });
  await expect(link).toBeFocused();
  await expect(link).toHaveCSS("transition-duration", "0s");
  await expect(link).toHaveCSS("outline-style", "solid");
});

test.describe("touch surfaces", () => {
  test.use({
    hasTouch: true,
    isMobile: true,
    viewport: { width: 402, height: 874 },
  });
  test("tap activates once without a persistent pointer-hover treatment", async ({
    page,
  }) => {
    await page.goto(
      "/iframe.html?id=chrome-interactive-surface--states&viewMode=story",
    );
    const action = page.getByRole("button", {
      name: "Open settings",
      exact: true,
    });
    const resting = await background(action);
    await action.tap();
    await expect(page.getByRole("status")).toHaveText("Actions: 1");
    await expect.poll(() => background(action)).toBe(resting);
  });
});
