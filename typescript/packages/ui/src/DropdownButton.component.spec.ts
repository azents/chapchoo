import { expect, test } from "@playwright/test";

test("an initially open controlled dropdown returns Escape focus to its trigger", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=controls-dropdown-button--opened&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: "Personal space", exact: true }),
  ).toBeFocused();
  await page.keyboard.press("Escape");
  const trigger = page.getByRole("button", {
    name: "Choose workspace",
    exact: true,
  });
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
  await expect(trigger).toBeFocused();
});

for (const touch of [false, true]) {
  test.describe(touch ? "touch dropdown" : "mouse dropdown", () => {
    test.use({
      hasTouch: touch,
      isMobile: touch,
      viewport: touch
        ? { width: 402, height: 874 }
        : { width: 1280, height: 900 },
    });
    test("same trigger toggles, outside dismisses, Escape returns focus", async ({
      page,
    }) => {
      await page.goto(
        "/iframe.html?id=controls-dropdown-button--interactive&viewMode=story",
      );
      const trigger = page.getByRole("button", {
        name: "Choose workspace",
        exact: true,
      });
      const activate = async (): Promise<void> => {
        if (touch) {
          await trigger.tap();
        } else {
          await trigger.click();
        }
      };
      await activate();
      await expect(trigger).toHaveAttribute("aria-expanded", "true");
      await activate();
      await expect(trigger).toHaveAttribute("aria-expanded", "false");
      await activate();
      const outside = page.getByRole("button", {
        name: "Outside action",
        exact: true,
      });
      if (touch) {
        await outside.tap();
      } else {
        await outside.click();
      }
      await expect(trigger).toHaveAttribute("aria-expanded", "false");
      await activate();
      await page.keyboard.press("Escape");
      await expect(trigger).toHaveAttribute("aria-expanded", "false");
      await expect(trigger).toBeFocused();
      await page.keyboard.press("Space");
      await expect(trigger).toHaveAttribute("aria-expanded", "true");
      await page.keyboard.press("Escape");
      await expect(trigger).toHaveAttribute("aria-expanded", "false");
    });
  });
}
