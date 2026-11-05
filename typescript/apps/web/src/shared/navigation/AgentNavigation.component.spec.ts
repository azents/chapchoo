import { expect, test } from "@playwright/test";

test("Team page shell renders shared Agent choices, current selection and bounded pages", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=navigation-team-agents--team-page&viewMode=story",
  );
  const sidebar = page.locator("aside");
  const selected = sidebar.getByRole("link", { name: /Corgi.*@corgi/ });
  await expect(selected).toHaveAttribute("aria-current", "page");
  await expect(selected).toHaveAttribute(
    "href",
    "/teams/design/agents/corgi/chat",
  );
  await expect(
    sidebar.getByRole("link", { name: "Create Agent", exact: true }),
  ).toHaveAttribute("href", "/teams/design/new-agent");
  await expect(
    sidebar.getByRole("link", { name: "Team settings", exact: true }),
  ).toBeVisible();
  await sidebar.getByRole("button", { name: "Next page", exact: true }).click();
  await expect(
    sidebar.getByRole("link", { name: /Bichon.*@bichon/ }),
  ).toBeVisible();
  await sidebar
    .getByRole("button", { name: "First page", exact: true })
    .click();
  await expect(selected).toBeVisible();
});

test("mobile Team navigation contains the same Agent choices", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=navigation-team-agents--team-page&viewMode=story",
  );
  await page
    .getByRole("button", { name: "Open navigation", exact: true })
    .click();
  const drawer = page.getByRole("dialog", { name: "Design team", exact: true });
  await expect(
    drawer.getByRole("link", { name: /Corgi.*@corgi/ }),
  ).toHaveAttribute("href", "/teams/design/agents/corgi/chat");
  await expect(
    drawer.getByRole("link", { name: "Create Agent", exact: true }),
  ).toBeVisible();
});
