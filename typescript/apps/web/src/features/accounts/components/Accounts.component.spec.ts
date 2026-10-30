import { expect, test } from "@playwright/test";
import messages from "../../../../messages/en-US.json" with { type: "json" };

test("reset completion component announces the app-owned exact message", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=accounts-authentication--reset-complete&viewMode=story",
  );
  await expect(page.getByRole("status")).toHaveText(
    messages.accounts.resetDone,
  );
  await expect(
    page.getByRole("link", { name: messages.accounts.backToLogin }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: messages.accounts.savePassword }),
  ).toHaveCount(0);
});

test("owner workspace component fits a 375-wide viewport", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/iframe.html?id=accounts-workspace--owner&viewMode=story");
  await expect(
    page.getByRole("heading", { name: "Design team", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
});

test("selected web typography and control surfaces are applied on narrow auth", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 850 });
  await page.goto(
    "/iframe.html?id=accounts-authentication--password-login&viewMode=story",
  );
  const email = page.getByRole("textbox", { name: messages.accounts.email });
  await expect(email).toHaveCSS("background-color", "rgb(255, 254, 250)");
  await expect(email).toHaveCSS("border-color", "rgb(112, 119, 101)");
  await expect(email).toHaveCSS("min-height", "44px");
  await expect(page.locator("body")).toHaveCSS(
    "font-family",
    "Pretendard, sans-serif",
  );
  await expect(
    page.getByRole("button", { name: messages.accounts.showPassword }),
  ).toHaveCSS("min-height", "44px");
});
