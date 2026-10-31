import { expect, test } from "@playwright/test";
import messages from "../../../../messages/en-US.json" with { type: "json" };

test("login keeps its server-selected character during form re-renders", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=accounts-authentication--server-selected-character&viewMode=story",
  );
  const image = page.locator('img[src$="/05-husky-216.webp"]');
  await expect(image).toBeVisible();
  await page
    .getByRole("textbox", { name: messages.accounts.email })
    .fill("typing@example.invalid");
  await page.locator('input[type="password"]').fill("typing-password");
  await expect(
    page.getByRole("textbox", { name: messages.accounts.email }),
  ).toHaveValue("typing@example.invalid");
  await expect(image).toBeVisible();
  await expect(page.locator('img[src*="/design-assets/"]')).toHaveCount(1);
});

test("supplementary invitation failure preserves primary contexts and actions", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=accounts-workspace--supplementary-unavailable&viewMode=story",
  );
  await expect(
    page.getByRole("heading", { name: "Your spaces", exact: true }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("main")
      .getByRole("link", { name: "Personal space", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Create team", exact: true }),
  ).toBeVisible();
});

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

test("auth footer links lead to other modes rather than the current screen", async ({
  page,
}) => {
  for (const [story, current] of [
    ["password-login", "/login"],
    ["request-signup", "/signup"],
    ["administrator-link-required", "/reset-password"],
  ]) {
    await page.goto(
      `/iframe.html?id=accounts-authentication--${story}&viewMode=story`,
    );
    await expect(page.locator("main")).toBeVisible();
    await expect(page.locator(`a[href="${current}"]`)).toHaveCount(0);
    await expect(page.locator("main a")).toHaveCount(2);
  }
});
