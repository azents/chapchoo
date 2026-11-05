import { test as base, expect } from "@playwright/test";
import { Assembly } from "@chapchoo/testenv/assembly";
import type { Page } from "@playwright/test";

const test = base.extend<object, { assembly: Assembly }>({
  assembly: [
    async ({ playwright }, use) => {
      void playwright;
      const assembly = await Assembly.create();
      try {
        await use(assembly);
      } finally {
        await assembly.close();
      }
    },
    { scope: "worker", timeout: 660_000 },
  ],
});

async function login(
  page: Page,
  url: string,
  email: string,
  password: string,
): Promise<void> {
  await page.goto(`${url}/login`);
  await page
    .getByRole("textbox", { name: /^(Email address|Email)$/ })
    .fill(email)
    .catch(() => {
      throw new Error("Computer acceptance login email field unavailable");
    });
  await page
    .getByLabel(/^Password\s*\*?$/)
    .fill(password)
    .catch(() => {
      throw new Error("Computer acceptance login password field unavailable");
    });
  await page.getByRole("button", { name: /^(Log in|Sign in)$/ }).click();
  await expect(
    page.getByRole("heading", { name: /^(Agents|Signup links)$/ }),
  ).toBeVisible();
}
async function fact(page: Page, label: string, value: string): Promise<void> {
  if (label === "Computer power") {
    await expect(
      page.getByText(`Computer · ${value}`, { exact: true }),
    ).toBeVisible({ timeout: 90_000 });
    return;
  }
  await page
    .getByRole("button", { name: "Computer details", exact: true })
    .click();
  const dialog = page.getByRole("dialog", {
    name: "Computer details",
    exact: true,
  });
  await expect(
    dialog
      .getByText(label, { exact: true })
      .locator("..")
      .getByText(value, { exact: true }),
  ).toBeVisible({ timeout: 90_000 });
  await dialog.getByRole("button", { name: "Close", exact: true }).click();
}

test("owner manages one Computer while Admin observes independent cleanup inventory", async ({
  assembly,
  browser,
}) => {
  test.setTimeout(300_000);
  await assembly.initializeAdministrator();
  await assembly.startComputerProvider();
  // Match account defaults; display-detection interaction belongs to account flows.
  const context = await browser.newContext({
    locale: "en-US",
    timezoneId: "UTC",
  });
  const operator = await browser.newContext({
    locale: "en-US",
    timezoneId: "UTC",
  });
  try {
    const owner = await context.newPage();
    await login(
      owner,
      assembly.generalUrl,
      assembly.administrator.email,
      assembly.administrator.password,
    );
    await owner.goto(`${assembly.generalUrl}/teams/my/agents`);
    const listUrl = owner.url();
    await owner.getByRole("link", { name: "New Agent", exact: true }).click();
    await expect(owner).toHaveURL(/\/teams\/my\/new-agent$/);
    await owner
      .getByRole("textbox", { name: "Agent name", exact: true })
      .fill("Acceptance companion")
      .catch(() => {
        throw new Error("Computer acceptance create form unavailable");
      });
    await owner
      .getByRole("combobox", { name: "Computer Provider", exact: true })
      .click();
    await owner
      .getByRole("option", { name: "Acceptance Kubernetes", exact: true })
      .click({ timeout: 90_000 });
    await owner
      .getByRole("group", { name: "Profile image", exact: true })
      .getByRole("button", { name: /^corgi$/i })
      .click();
    await owner
      .getByRole("group", { name: "Color", exact: true })
      .getByRole("button", { name: "Rose", exact: true })
      .click();
    await owner
      .getByRole("button", { name: "Create Agent", exact: true })
      .click();
    await expect(owner).toHaveURL(/\/teams\/my\/agents\/[a-z0-9-]+\/chat$/);
    const chatUrl = owner.url();
    await owner.setViewportSize({ width: 390, height: 844 });
    await owner
      .getByRole("button", { name: "Agents and conversations", exact: true })
      .click();
    const navigation = owner.getByRole("dialog", {
      name: "Agents and conversations",
      exact: true,
    });
    const switchWorkspace = navigation.getByRole("button", {
      name: "Switch workspace",
      exact: true,
    });
    await switchWorkspace.click();
    await expect(switchWorkspace).toHaveAttribute("aria-expanded", "true");
    await owner.keyboard.press("Escape");
    await expect(navigation).toBeVisible();
    await expect(switchWorkspace).toHaveAttribute("aria-expanded", "false");
    await navigation
      .getByRole("button", { name: "Account and settings", exact: true })
      .click();
    await expect(
      owner.getByRole("dialog", { name: "Account and settings", exact: true }),
    ).toBeVisible();
    await owner.keyboard.press("Escape");
    await expect(navigation).toBeVisible();
    await expect(
      navigation.getByRole("button", {
        name: "Account and settings",
        exact: true,
      }),
    ).toBeFocused();
    await navigation
      .getByRole("button", { name: "Close", exact: true })
      .click();
    await expect(
      owner.getByRole("button", {
        name: "Agents and conversations",
        exact: true,
      }),
    ).toBeFocused();
    await owner.setViewportSize({ width: 1280, height: 900 });
    await owner
      .getByRole("button", { name: /Acceptance companion.*Activity/ })
      .click();
    await fact(owner, "Computer power", "Running");
    await owner.getByRole("button", { name: "Stop", exact: true }).click();
    await owner
      .getByRole("dialog", { name: "Stop", exact: true })
      .getByRole("button", { name: "Cancel", exact: true })
      .click();
    await owner
      .getByRole("dialog", { name: "Activity", exact: true })
      .getByRole("button", { name: "Close", exact: true })
      .click();
    await owner
      .getByRole("link", {
        name: "Acceptance companion · Agent details",
        exact: true,
      })
      .click();
    await expect(
      owner.getByRole("heading", { name: "Acceptance companion", exact: true }),
    ).toBeVisible();
    await fact(owner, "Computer power", "Running");
    await expect(
      owner
        .getByRole("main")
        .locator('img[src$="/design-assets/03-corgi-216.webp"]'),
    ).toHaveCount(1);
    await fact(owner, "Computer connection", "Connected");
    const detailUrl = owner.url();
    await expect(owner).toHaveURL(/\/teams\/my\/agents\/[a-z0-9-]+$/);
    await owner
      .getByRole("button", { name: "Edit profile", exact: true })
      .click();
    const profile = owner.getByRole("dialog", {
      name: "Edit profile",
      exact: true,
    });
    await profile
      .getByRole("group", { name: "Profile image", exact: true })
      .getByRole("button", { name: /^retriever$/i })
      .click();
    await profile
      .getByRole("group", { name: "Color", exact: true })
      .getByRole("button", { name: "Butter", exact: true })
      .click();
    await owner
      .getByRole("textbox", { name: "Name", exact: true })
      .fill("Saved companion");
    await owner.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      owner.getByRole("heading", { name: "Saved companion", exact: true }),
    ).toBeVisible();
    const document = await context.newPage();
    await document.route("**/*", (route) =>
      route.request().resourceType() === "script"
        ? route.abort()
        : route.continue(),
    );
    await document.goto(detailUrl);
    await expect(
      document.getByRole("heading", {
        name: "Saved companion",
        exact: true,
      }),
    ).toBeVisible();
    await expect(
      document
        .getByRole("main")
        .locator('img[src$="/design-assets/01-retriever-216.webp"]'),
    ).toHaveCount(1);
    await document.close();
    await owner.goto(listUrl);
    await owner
      .getByRole("link", { name: "Saved companion", exact: true })
      .click();
    await expect(owner).toHaveURL(chatUrl);
    await owner
      .getByRole("link", {
        name: "Saved companion · Agent details",
        exact: true,
      })
      .click();

    const shutdown = assembly.pauseComputerShutdown();
    try {
      await owner.getByRole("button", { name: "Stop", exact: true }).click();
      await owner
        .getByRole("dialog", { name: "Stop", exact: true })
        .getByRole("button", { name: "Confirm", exact: true })
        .click();
      await shutdown.started;
      await owner.getByRole("button", { name: "Start", exact: true }).click();
      await expect(
        owner.getByRole("button", {
          name: "Restarting",
          exact: true,
        }),
      ).toBeDisabled();
      await owner.reload();
      await expect(
        owner.getByRole("button", {
          name: "Restarting",
          exact: true,
        }),
      ).toBeDisabled();
      expect(assembly.executionAllocations).toBe(1);
      expect(assembly.savedAllocations).toBe(1);
    } finally {
      shutdown.release();
    }
    await fact(owner, "Computer power", "Running");
    expect(assembly.executionAllocations).toBe(2);
    await fact(owner, "Computer connection", "Connected");
    expect(assembly.savedAllocations).toBe(1);

    const admin = await operator.newPage();
    await login(
      admin,
      assembly.adminUrl,
      assembly.administrator.email,
      assembly.administrator.password,
    );
    await admin.goto(`${assembly.adminUrl}/computer-providers`);
    await admin
      .getByRole("link", { name: "Acceptance Kubernetes", exact: true })
      .click();
    await expect(
      admin.getByRole("main").getByText("Read-only inventory", { exact: true }),
    ).toBeVisible();
    await expect(
      admin.getByText("Saved companion", { exact: true }),
    ).toHaveCount(0);
    const inventoryUrl = admin.url();
    await admin
      .getByRole("link", { name: "Managed resources", exact: true })
      .click();
    await expect(
      admin.getByRole("heading", { name: "Managed resources", exact: true }),
    ).toBeVisible();
    await expect(admin.getByText(/^UID: /).first()).toBeVisible();
    await admin.goto(inventoryUrl);
    await owner
      .getByRole("button", { name: "Delete Agent", exact: true })
      .click();
    await owner
      .getByRole("dialog")
      .getByRole("button", { name: "Confirm", exact: true })
      .click();
    await expect(owner).toHaveURL(listUrl, {
      timeout: 90_000,
    });
    await expect(
      owner.getByText("No Agents yet.", { exact: true }),
    ).toBeVisible();
    await expect(admin.getByText(/Cleanup confirmed/)).toBeVisible({
      timeout: 90_000,
    });
    expect(assembly.savedAllocations).toBe(1);
  } finally {
    await context.close();
    await operator.close();
  }
});
