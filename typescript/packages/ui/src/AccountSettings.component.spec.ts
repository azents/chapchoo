import { expect, test } from "@playwright/test";

for (const [story, width] of [
  ["interactive", 1280],
  ["mobile-interactive", 390],
] as const) {
  test(`account settings ${story} presents one account entry and restores focus`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(
      `/iframe.html?id=chrome-account-settings--${story}&viewMode=story`,
    );
    const trigger = page.getByRole("button", {
      name: "Account and settings",
      exact: true,
    });
    await trigger.click();
    await expect(
      page.getByText("companion@example.invalid", { exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Profile", exact: true }),
    ).toHaveAttribute("href", "/profile");
    await expect(
      page.getByRole("link", { name: "Profile", exact: true }),
    ).toHaveAttribute("target", "_blank");
    await expect(
      page.getByRole("link", { name: "Profile", exact: true }),
    ).toHaveAttribute("rel", "noopener");
    await expect(
      page
        .getByRole("link", { name: "Profile", exact: true })
        .getByText("companion@example.invalid", { exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Sign out", exact: true }),
    ).toHaveCount(1);
    for (const label of ["Language", "Time zone", "Regional format"]) {
      await expect(
        page.getByRole("combobox", { name: label, exact: true }),
      ).toBeVisible();
    }
    await page.keyboard.press("Escape");
    await expect(
      page.getByText("companion@example.invalid", { exact: true }),
    ).not.toBeVisible();
    await expect(trigger).toBeFocused();
    await trigger.click();
    await page
      .getByRole("link", { name: "Profile", exact: true })
      .press("Escape");
    await expect(
      page.getByText("companion@example.invalid", { exact: true }),
    ).not.toBeVisible();
    await expect(trigger).toBeFocused();
  });
}

test("account menu retains real save and retry feedback", async ({ page }) => {
  await page.goto(
    "/iframe.html?id=chrome-account-settings--saved&viewMode=story",
  );
  await expect(page.getByRole("status")).toHaveText("Saved");
  await page.goto(
    "/iframe.html?id=chrome-account-settings--failed-save&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: "Retry", exact: true }),
  ).toBeVisible();
});
