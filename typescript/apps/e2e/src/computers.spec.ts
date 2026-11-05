import { test as base, expect } from "@playwright/test";
import { Assembly } from "@chapchoo/testenv/assembly";
import type { APIResponse, Page, Response } from "@playwright/test";

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

async function browserData(response: Response | APIResponse): Promise<object> {
  const payload: unknown = await response.json();
  if (
    typeof payload !== "object" ||
    payload === null ||
    !("result" in payload) ||
    typeof payload.result !== "object" ||
    payload.result === null ||
    !("data" in payload.result) ||
    typeof payload.result.data !== "object" ||
    payload.result.data === null ||
    !("json" in payload.result.data) ||
    typeof payload.result.data.json !== "object" ||
    payload.result.data.json === null
  )
    throw new Error("Browser RPC response unavailable");
  return payload.result.data.json;
}

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
    // Read the public browser boundary, not database state or framework internals.
    const routeRead = owner.waitForRequest((request) =>
      request.url().includes("/api/trpc/computers.getInTeam"),
    );
    await owner
      .getByRole("button", { name: "Create Agent", exact: true })
      .click();
    await expect(owner).toHaveURL(/\/teams\/my\/agents\/[a-z0-9-]+\/chat$/);
    const chatUrl = owner.url();
    const input: unknown = (await routeRead).postDataJSON();
    if (
      typeof input !== "object" ||
      input === null ||
      !("json" in input) ||
      typeof input.json !== "object" ||
      input.json === null ||
      !("teamId" in input.json) ||
      typeof input.json.teamId !== "string"
    )
      throw new Error("Agent Team query unavailable");
    const inventory = await browserData(
      await context.request.post(
        `${assembly.generalUrl}/api/trpc/computers.list`,
        {
          headers: { origin: new URL(assembly.generalUrl).origin },
          data: { json: { teamId: input.json.teamId } },
        },
      ),
    );
    const item: unknown =
      "items" in inventory && Array.isArray(inventory.items)
        ? inventory.items[0]
        : null;
    if (
      typeof item !== "object" ||
      item === null ||
      !("id" in item) ||
      typeof item.id !== "string"
    )
      throw new Error("Agent inventory ID unavailable");
    const agentId = item.id;
    const pagedChatUrl = `${chatUrl}?agentAfter=${encodeURIComponent(agentId)}`;
    const pagedInventory = owner
      .waitForResponse(
        (response) =>
          response.url().includes("/api/trpc/computers.list") &&
          Boolean(response.request().postData()?.includes(agentId)),
      )
      .then(async (response) => {
        expect(response.request().postData()).toContain(`"after":"${agentId}"`);
        return browserData(response);
      });
    await owner.goto(pagedChatUrl);
    const paged = await pagedInventory;
    if (!("items" in paged))
      throw new Error("Paged Agent inventory unavailable");
    expect(paged.items).toEqual([]);
    const sidebar = owner.locator("aside");
    const selectedAgent = sidebar.getByRole("link", {
      name: /Acceptance companion.*@/,
    });
    // The only Agent is outside the selected inventory page but remains selected.
    await expect(selectedAgent).toHaveAttribute("aria-current", "page");
    await expect(selectedAgent).toHaveAttribute(
      "href",
      new URL(pagedChatUrl).pathname + new URL(pagedChatUrl).search,
    );
    await owner.reload();
    await expect(selectedAgent).toHaveAttribute("aria-current", "page");
    await sidebar
      .getByRole("link", { name: "Create Agent", exact: true })
      .click();
    await expect(owner).toHaveURL(
      new URL(pagedChatUrl).origin +
        new URL(pagedChatUrl).pathname.replace(
          /\/agents\/[^/]+\/chat$/,
          "/new-agent",
        ) +
        new URL(pagedChatUrl).search,
    );
    await owner
      .getByRole("link", { name: "Back to Agents", exact: true })
      .click();
    await expect(owner).toHaveURL(
      `${listUrl}?agentAfter=${encodeURIComponent(agentId)}`,
    );
    await owner
      .getByRole("main")
      .getByRole("link", { name: "Acceptance companion", exact: true })
      .click();
    await expect(owner).toHaveURL(pagedChatUrl);
    await sidebar
      .getByRole("button", { name: "First page", exact: true })
      .click();
    await expect(owner).toHaveURL(chatUrl);
    await owner.goBack();
    await expect(owner).toHaveURL(pagedChatUrl);
    await expect(
      sidebar.getByRole("button", { name: "First page", exact: true }),
    ).toBeVisible();
    await owner.goForward();
    await expect(owner).toHaveURL(chatUrl);
    await expect(
      sidebar.getByRole("button", { name: "First page", exact: true }),
    ).toHaveCount(0);
    await owner.setViewportSize({ width: 390, height: 844 });
    await owner
      .getByRole("button", { name: "Agents and conversations", exact: true })
      .click();
    const navigation = owner.getByRole("dialog", {
      name: "Agents and conversations",
      exact: true,
    });
    await expect(
      navigation.getByRole("link", { name: /Acceptance companion.*@/ }),
    ).toHaveAttribute("href", new URL(chatUrl).pathname);
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
      owner.getByRole("main").getByText("No Agents yet.", { exact: true }),
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
