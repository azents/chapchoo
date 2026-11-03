import { safeMessage } from "@chapchoo/testenv/safety";
import { test, expect } from "./fixture.js";
import type { Assembly } from "./fixture.js";
import type { FixtureUser } from "@chapchoo/testenv/safety";
import type { Browser, BrowserContext, Page } from "@playwright/test";

const contexts: BrowserContext[] = [];
const websiteErrors: string[] = [];
let administrator: Page;
let member: Page;
let second: Page;
let sharedGeneral: Page;

async function isolatedPage(browser: Browser): Promise<Page> {
  const context = await browser.newContext({
    locale: "en-US",
    timezoneId: "Asia/Seoul",
    colorScheme: "light",
    deviceScaleFactor: 1,
    viewport: { width: 1280, height: 900 },
  });
  contexts.push(context);
  const page = await context.newPage();
  page.on("pageerror", (error) =>
    websiteErrors.push(safeMessage(error.message)),
  );
  page.on("console", (message) => {
    if (
      message.type() === "error" &&
      /hydration|hydrating|did not match|server.rendered|react.dev\/errors\/418/i.test(
        message.text(),
      )
    )
      websiteErrors.push("React reported a hydration mismatch");
  });
  return page;
}
async function login(
  page: Page,
  url: string,
  user: FixtureUser,
): Promise<void> {
  await page.goto(`${url}/login`);
  await page
    .getByRole("textbox", { name: "Email address", exact: true })
    .fill(user.email);
  await page.getByLabel(/^Password\s*\*?$/).fill(user.password);
  await page.getByRole("button", { name: "Log in", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Agents", exact: true }),
  ).toBeVisible();
}
async function logout(page: Page): Promise<void> {
  await page
    .getByRole("button", { name: "Account and settings", exact: true })
    .click();
  await page.getByRole("button", { name: "Sign out", exact: true }).click();
}
async function serverContent(
  context: BrowserContext,
  url: string,
  heading: string,
  text?: string,
): Promise<void> {
  const page = await context.newPage();
  try {
    await page.route("**/*", (route) =>
      route.request().resourceType() === "script"
        ? route.abort()
        : route.continue(),
    );
    await page.goto(url);
    await expect(
      page.getByRole("heading", { name: heading, exact: true }),
    ).toBeVisible();
    if (text)
      await expect(
        page.getByRole("main").getByText(text, { exact: true }).first(),
      ).toBeVisible();
  } finally {
    await page.close();
  }
}
/** Retain the real document/CSS-arrival boundary, not a theme-value matrix. */
async function serverCanvas(
  context: BrowserContext,
  url: string,
): Promise<void> {
  const page = await context.newPage();
  let release = (): void => {};
  const styles = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route("**/*", async (route) => {
    if (route.request().resourceType() === "script") {
      await route.abort();
      return;
    }
    if (route.request().resourceType() === "stylesheet") {
      await styles;
    }
    await route.continue();
  });
  try {
    await page.goto(url, { waitUntil: "commit" });
    await expect(page.locator("html")).toHaveAttribute(
      "data-mantine-color-scheme",
      "light",
    );
    await expect(
      page.locator('head meta[name="color-scheme"]'),
    ).toHaveAttribute("content", "light dark");
    expect(
      await page.evaluate(
        () => getComputedStyle(document.documentElement).backgroundColor,
      ),
    ).toBe("rgb(253, 252, 248)");
    release();
    await page.waitForLoadState("load");
    await expect(page.locator("body")).toHaveCSS(
      "background-color",
      "rgb(253, 252, 248)",
    );
    await expect(page.locator("form")).toBeVisible();
  } finally {
    release();
    await page.close();
  }
}

async function issuedLink(
  page: Page,
  assembly: Assembly,
  kind: "signup" | "reset-password",
): Promise<string> {
  const field = page.getByRole("textbox", { name: "New link", exact: true });
  await expect(field).toBeVisible();
  const value = await field.inputValue();
  let valid = false;
  try {
    const url = new URL(value);
    const expected = new URL(assembly.generalUrl);
    valid =
      url.origin === expected.origin &&
      url.pathname === `${expected.pathname.replace(/\/$/, "")}/${kind}` &&
      /^[A-Za-z0-9_-]{43}$/.test(url.searchParams.get("token") ?? "");
  } catch {
    /* Never include a proof-bearing URL in assertion diagnostics. */
  }
  expect(valid, "Issued link must target the configured General mount").toBe(
    true,
  );
  await page.getByRole("button", { name: "Dismiss", exact: true }).click();
  return value;
}
async function confirm(page: Page): Promise<void> {
  const dialog = page.getByRole("dialog", {
    name: "Confirm this change",
    exact: true,
  });
  await dialog.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(dialog).not.toBeVisible();
}

test.describe("Assembled account and team acceptance", () => {
  test.describe.configure({ mode: "serial" });
  test.beforeAll(async ({ browser }) => {
    administrator = await isolatedPage(browser);
    member = await isolatedPage(browser);
    second = await isolatedPage(browser);
  });
  test.afterEach(() => {
    if (websiteErrors.length)
      throw new Error(
        `Website exception: ${websiteErrors.shift() ?? "Unexpected browser error"}`,
      );
  });
  test.afterAll(async () => {
    const results = await Promise.allSettled(
      contexts.map((context) => context.close()),
    );
    if (results.some((result) => result.status === "rejected"))
      throw new Error("Owned browser context cleanup failed");
  });

  test("administrator bootstrap, issued signup and shared identity reach a private personal context", async ({
    assembly,
  }) => {
    await serverCanvas(administrator.context(), `${assembly.generalUrl}/login`);
    await administrator.goto(`${assembly.adminUrl}/setup`);
    await administrator
      .getByRole("textbox", { name: "Email", exact: true })
      .fill(assembly.administrator.email);
    await administrator
      .getByLabel(/^Setup proof\s*\*?$/)
      .fill(assembly.setupProof);
    await administrator
      .getByLabel(/^Password\s*\*?$/)
      .fill(assembly.administrator.password);
    await administrator
      .getByRole("button", { name: "Create administrator", exact: true })
      .click();
    await expect(
      administrator.getByRole("heading", { name: "Signup links", exact: true }),
    ).toBeVisible();
    await administrator
      .locator("aside")
      .getByRole("button", { name: "Switch workspace", exact: true })
      .click();
    await administrator
      .getByRole("button", { name: "Create team", exact: true })
      .click();
    const creation = administrator.getByRole("dialog", {
      name: "Create team",
      exact: true,
    });
    await creation
      .getByRole("textbox", { name: "Team name", exact: true })
      .fill("Administrator workspace");
    await creation
      .getByRole("textbox", { name: "Handle", exact: true })
      .fill(`${assembly.teamHandle}-admin`);
    await creation.getByRole("button", { name: "Create", exact: true }).click();
    await expect(
      administrator.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    await administrator
      .locator("aside")
      .getByRole("button", { name: "Switch workspace", exact: true })
      .click();
    await administrator
      .getByRole("link", { name: "Administration", exact: true })
      .click();
    await expect(
      administrator.getByRole("heading", { name: "Signup links", exact: true }),
    ).toBeVisible();
    sharedGeneral = await administrator.context().newPage();
    await sharedGeneral.goto(`${assembly.generalUrl}/`);
    await expect(
      sharedGeneral.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    await expect(sharedGeneral).toHaveURL(/\/agents\?team=[^&]+$/);
    await administrator
      .getByRole("textbox", { name: "Recipient email", exact: true })
      .fill(assembly.member.email);
    await administrator
      .getByRole("button", { name: "Create link", exact: true })
      .click();
    const signup = await issuedLink(administrator, assembly, "signup");
    await serverContent(
      administrator.context(),
      `${assembly.adminUrl}/`,
      "Signup links",
      assembly.member.email,
    );
    await member.goto(signup);
    await member
      .getByRole("textbox", { name: "Email address", exact: true })
      .fill(assembly.member.email);
    await member.getByLabel(/^Password\s*\*?$/).fill(assembly.member.password);
    await member
      .getByRole("button", { name: "Save password", exact: true })
      .click();
    await expect(
      member.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    await expect(
      member
        .locator("aside")
        .getByRole("button", { name: "Switch workspace", exact: true }),
    ).toContainText("Your space");
    await serverContent(
      member.context(),
      member.url(),
      "Agents",
      "No Agents yet.",
    );
    await member.goto(`${assembly.generalUrl}/profile`);
    const zone = member
      .getByRole("main")
      .getByRole("combobox", { name: "Time zone", exact: true });
    await zone.click();
    await member
      .getByRole("option", { name: "America/New_York", exact: true })
      .click();
    await expect(
      member.getByRole("main").getByText("Applied", { exact: true }),
    ).toBeVisible();
    await member.reload();
    await expect(zone).toHaveValue("America/New_York");
    await serverContent(
      member.context(),
      `${assembly.generalUrl}/profile`,
      "Profile",
      assembly.member.email,
    );
    const cookie = (await member.context().cookies()).find(
      (item) => item.name === "chapchoo-general-session",
    );
    expect(Boolean(cookie), "Shared host session is present").toBe(true);
    expect(cookie?.httpOnly).toBe(true);
    expect(cookie?.path).toBe("/");
  });

  test("workspace creation, invitation acceptance, ownership transfer and leave preserve context authority", async ({
    assembly,
  }) => {
    await assembly.seedSecondUser();
    await member.goto(`${assembly.generalUrl}/`);
    await member
      .locator("aside")
      .getByRole("button", { name: "Switch workspace", exact: true })
      .click();
    await member
      .getByRole("button", { name: "Create team", exact: true })
      .click();
    const creation = member.getByRole("dialog", {
      name: "Create team",
      exact: true,
    });
    await creation
      .getByRole("textbox", { name: "Team name", exact: true })
      .fill(assembly.teamName);
    await creation
      .getByRole("textbox", { name: "Handle", exact: true })
      .fill(assembly.teamHandle);
    await creation.getByRole("button", { name: "Create", exact: true }).click();
    await expect(creation).not.toBeVisible();
    await expect(
      member
        .locator("aside")
        .getByRole("button", { name: "Switch workspace", exact: true }),
    ).toContainText(assembly.teamName);
    const agentUrl = member.url();
    await member
      .locator("aside")
      .getByRole("link", { name: "Team settings", exact: true })
      .click();
    const teamUrl = member.url();
    await serverContent(
      member.context(),
      teamUrl,
      assembly.teamName,
      assembly.member.email,
    );
    await member
      .getByRole("textbox", { name: "Email address", exact: true })
      .fill(assembly.second.email);
    await member
      .getByRole("button", { name: "Invite by email", exact: true })
      .click();
    await expect(
      member.getByText(assembly.second.email, { exact: true }),
    ).toBeVisible();
    await login(second, assembly.generalUrl, assembly.second);
    await second
      .locator("aside")
      .getByRole("link", { name: "Invitations", exact: true })
      .click();
    await expect(
      second.getByRole("heading", { name: "Invitations for you", exact: true }),
    ).toBeVisible();
    await second.getByRole("button", { name: "Accept", exact: true }).click();
    await second
      .locator("aside")
      .getByRole("button", { name: "Switch workspace", exact: true })
      .click();
    await second
      .getByRole("link", { name: assembly.teamName, exact: true })
      .click();
    await expect(second).toHaveURL(agentUrl);
    await second
      .locator("aside")
      .getByRole("link", { name: "Team settings", exact: true })
      .click();
    await expect(
      second.getByRole("button", { name: "Delete team", exact: true }),
    ).toHaveCount(0);
    await member.reload();
    const target = member
      .getByRole("row")
      .filter({ hasText: assembly.second.email });
    await target.getByRole("button", { name: "Actions", exact: true }).click();
    await member
      .getByRole("menuitem", { name: "Transfer ownership", exact: true })
      .click();
    await confirm(member);
    await expect(
      target.getByRole("cell", { name: "Owner", exact: true }),
    ).toBeVisible();
    await member
      .getByRole("button", { name: "Leave team", exact: true })
      .click();
    await confirm(member);
    await expect(
      member.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    await expect(
      member
        .locator("aside")
        .getByRole("button", { name: "Switch workspace", exact: true }),
    ).toContainText("Your space");
    await second.goto(teamUrl);
    await expect(
      second
        .getByRole("row")
        .filter({ hasText: assembly.second.email })
        .getByRole("cell", { name: "Owner", exact: true }),
    ).toBeVisible();
    await expect(
      second.getByRole("row").filter({ hasText: assembly.member.email }),
    ).toHaveCount(0);
  });

  test("reset revokes old sessions, Admin denial preserves General, and shared logout withdraws both surfaces", async ({
    assembly,
    browser,
  }) => {
    await administrator.goto(`${assembly.adminUrl}/password-resets`);
    await administrator
      .getByRole("textbox", { name: "Recipient email", exact: true })
      .fill(assembly.member.email);
    await administrator
      .getByRole("button", { name: "Create link", exact: true })
      .click();
    const reset = await issuedLink(administrator, assembly, "reset-password");
    const resetPage = await isolatedPage(browser);
    await resetPage.goto(reset);
    await resetPage.getByLabel(/^Password\s*\*?$/).fill(assembly.resetPassword);
    await resetPage
      .getByRole("button", { name: "Save password", exact: true })
      .click();
    await expect(resetPage.getByRole("status")).toBeVisible();
    expect(
      (await resetPage.context().cookies()).some(
        (cookie) => cookie.name === "chapchoo-general-session",
      ),
      "Reset does not authenticate the recipient",
    ).toBe(false);
    await member.goto(`${assembly.generalUrl}/`);
    await expect(member).toHaveURL(`${assembly.generalUrl}/login`);
    await login(member, assembly.generalUrl, {
      email: assembly.member.email,
      password: assembly.resetPassword,
    });
    const denied = await second
      .context()
      .request.post(`${assembly.adminUrl}/api/trpc/auth.identity`, {
        headers: { origin: new URL(assembly.adminUrl).origin },
        data: { json: null },
      });
    expect(denied.status()).toBe(403);
    expect(
      Boolean(denied.headers()["set-cookie"]?.includes("Max-Age=0")),
      "Admin denial does not clear the General session",
    ).toBe(false);
    await second.goto(`${assembly.generalUrl}/`);
    await expect(
      second.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    await logout(second);
    await expect(second).toHaveURL(`${assembly.generalUrl}/login`);
    await administrator.goto(`${assembly.adminUrl}/`);
    await logout(administrator);
    await expect(administrator).toHaveURL(`${assembly.adminUrl}/login`);
    await sharedGeneral.reload();
    await expect(sharedGeneral).toHaveURL(`${assembly.generalUrl}/login`);
    await member.goto(`${assembly.generalUrl}/`);
    await expect(
      member.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
  });
});
