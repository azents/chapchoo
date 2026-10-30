import { expect, test } from "@playwright/test";
import messages from "../../../../messages/en-US.json" with { type: "json" };

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
