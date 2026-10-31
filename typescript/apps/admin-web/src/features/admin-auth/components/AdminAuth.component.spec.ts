import { expect, test } from "@playwright/test";
import messages from "../../../../messages/en-US.json" with { type: "json" };

test("Admin login keeps its server-selected character during form re-renders", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=admin-authentication--server-selected-character&viewMode=story",
  );
  const image = page.locator('img[src$="/06-border-collie-216.webp"]');
  await expect(image).toBeVisible();
  await page.locator('input[type="email"]').fill("typing@example.invalid");
  await page.locator('input[type="password"]').fill("typing-password");
  await expect(page.locator('input[type="email"]')).toHaveValue(
    "typing@example.invalid",
  );
  await expect(image).toBeVisible();
  await expect(page.locator('img[src*="/design-assets/"]')).toHaveCount(1);
});

test("rejected Admin form renders the exact permission guidance", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=admin-authentication--rejected&viewMode=story",
  );
  await expect(page.getByRole("alert")).toHaveText(
    messages.Errors.permissionDenied,
  );
  await expect(
    page.getByRole("button", { name: messages.AdminAuth.login }),
  ).toBeVisible();
});
