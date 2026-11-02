import { expect, test } from "@playwright/test";

test("creation dialog owns explicit cancel, pending admission and uncertain-outcome feedback", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--creation&viewMode=story",
  );
  const dialog = page.getByRole("dialog", { name: "Create team", exact: true });
  await expect(
    dialog.getByRole("textbox", { name: "Team name", exact: true }),
  ).toHaveValue("Design team");
  await expect(
    dialog.getByRole("textbox", { name: "Handle", exact: true }),
  ).toHaveValue("design");
  await expect(
    dialog.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeEnabled();
  await expect(
    dialog.getByRole("button", { name: "Create", exact: true }),
  ).toBeEnabled();
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--creation-pending&viewMode=story",
  );
  await expect(
    dialog.getByRole("textbox", { name: "Team name", exact: true }),
  ).toBeDisabled();
  await expect(
    dialog.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeDisabled();
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--creation-error&viewMode=story",
  );
  await expect(dialog.getByRole("alert")).toContainText(
    "Check the team list before submitting again.",
  );
});

test("mobile picker stays inside the left navigation Drawer with the existing companion", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--mobile-picker&viewMode=story",
  );
  const dialog = page.getByRole("dialog");
  await expect(
    dialog.getByRole("link", { name: "Your space", exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByRole("link", { name: "Design team", exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByRole("button", { name: "Create team", exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByRole("button", {
      name: "Replay character animation",
      exact: true,
    }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});

test("Admin chrome keeps Administration selected and shares the General logo destination", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--administration&viewMode=story",
  );
  await expect(
    page
      .getByRole("banner")
      .getByRole("link", { name: "chapchoo", exact: true }),
  ).toHaveAttribute("href", "/");
  await expect(
    page.getByRole("banner").getByText("admin", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Administration", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { name: "Your space", exact: true }),
  ).not.toHaveAttribute("aria-current", "page");
  await expect(
    page
      .locator("aside")
      .getByRole("link", { name: "Server settings", exact: true }),
  ).toBeVisible();
});
