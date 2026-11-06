import { authenticationCopy } from "@chapchoo/web-core/authentication-messages";
import { expect, test } from "@playwright/test";
const copy = authenticationCopy("ko");
test("saved reference row opens an explicit picker and blocks unchanged selection", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-agent-selection--saved&viewMode=story",
  );
  await expect(
    page
      .locator("#storybook-root")
      .getByText("건우의 ChatGPT", { exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: copy.change, exact: true }).click();
  const dialog = page.getByRole("dialog", { name: copy.choose, exact: true });
  await expect(
    dialog.getByRole("button", { name: copy.selected, exact: true }),
  ).toBeDisabled();
  await dialog.getByRole("button", { name: copy.close, exact: true }).click();
  await expect(
    page
      .locator("#storybook-root")
      .getByText("건우의 ChatGPT", { exact: true }),
  ).toBeVisible();
});
test("unselected and withdrawn identities do not appear authenticated", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-agent-selection--unselected&viewMode=story",
  );
  await expect(page.getByText(copy.unselected, { exact: true })).toBeVisible();
  await page.goto(
    "/iframe.html?id=authentication-agent-selection--withdrawn&viewMode=story",
  );
  await expect(page.getByText(copy.unavailable, { exact: true })).toBeVisible();
  await expect(
    page
      .locator("#storybook-root")
      .getByText("건우의 ChatGPT", { exact: true }),
  ).toHaveCount(0);
});
