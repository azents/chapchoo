import { mkdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { test, expect } from "./fixture.js";
import { safeMessage } from "@chapchoo/testenv/safety";
import type { Assembly } from "./fixture.js";
import type { FixtureUser } from "@chapchoo/testenv/safety";
import type { Browser, BrowserContext, Page } from "@playwright/test";

const contexts: BrowserContext[] = [];
const websiteErrors: string[] = [];
let administrator: Page;
let member: Page;
let second: Page;

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
    ) {
      websiteErrors.push("React reported an appearance hydration mismatch");
    }
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
    page.getByRole("heading", { name: "Your spaces", exact: true }),
  ).toBeVisible();
}

async function verifyInitialCanvas(
  context: BrowserContext,
  url: string,
  expected: {
    serverMode: "light" | "dark";
    mode: "light" | "dark";
    media: "light" | "dark";
  } = { serverMode: "dark", mode: "dark", media: "light" },
): Promise<void> {
  const page = await context.newPage();
  await page.emulateMedia({ colorScheme: expected.media });
  const background =
    expected.mode === "dark" ? "rgb(24, 28, 24)" : "rgb(253, 252, 248)";
  let releaseStyles: () => void = () => {};
  const stylesReady = new Promise<void>((resolve) => {
    releaseStyles = resolve;
  });
  await page.route("**/*", async (route) => {
    if (route.request().resourceType() === "script") {
      await route.abort();
      return;
    }
    if (route.request().resourceType() === "stylesheet") await stylesReady;
    await route.continue();
  });
  try {
    const response = await page.goto(url, { waitUntil: "commit" });
    if (!response) throw new Error("Initial document response is required");
    await expect(page.locator("html")).toHaveAttribute(
      "data-mantine-color-scheme",
      expected.serverMode,
    );
    // Parser-blocking styles keep the head bootstrap from running here.
    const canvas = await page.evaluate(() => {
      const root = document.documentElement;
      return {
        scheme: getComputedStyle(root).colorScheme,
        background: getComputedStyle(root).backgroundColor,
        inlineScheme: root.style.colorScheme,
        preference: root.dataset.colorModePreference,
      };
    });
    const scheme =
      canvas.preference === "system" ? "light dark" : expected.mode;
    expect(canvas.scheme).toBe(scheme);
    expect(canvas.inlineScheme).toBe(scheme);
    expect(canvas.background).toBe(background);
    const html = await response.text();
    expect(
      html.includes("<form"),
      "The initial HTML must contain the form",
    ).toBe(true);
    const meta = html.indexOf(`<meta name="color-scheme" content="${scheme}"`);
    const bootstrap = html.indexOf("<script>");
    expect(
      meta,
      "The initial document must advertise its scheme",
    ).toBeGreaterThanOrEqual(0);
    expect(
      meta,
      "Scheme metadata must precede the CSS-dependent bootstrap",
    ).toBeLessThan(bootstrap);
    await expect(
      page.locator('head meta[name="color-scheme"]'),
    ).toHaveAttribute("content", scheme);
    releaseStyles();
    await page.waitForLoadState("load");
    // External JavaScript is still blocked: SSR CSS must paint the body.
    await expect(page.locator("body")).toHaveCSS(
      "background-color",
      background,
    );
    await expect(page.locator("form")).toBeVisible();
    await expect(page.locator('input[type="email"]')).toBeVisible();
  } finally {
    releaseStyles();
    await page.close();
  }
}

async function issuedLink(
  page: Page,
  fixture: Assembly,
  kind: "signup" | "reset-password",
): Promise<string> {
  const field = page.getByRole("textbox", { name: "New link", exact: true });
  await expect(field).toBeVisible();
  const value = await field.inputValue();
  let valid = false;
  try {
    const url = new URL(value);
    const expected = new URL(fixture.generalUrl);
    valid =
      url.origin === expected.origin &&
      url.pathname === `${expected.pathname.replace(/\/$/, "")}/${kind}` &&
      /^[A-Za-z0-9_-]{43}$/.test(url.searchParams.get("token") ?? "");
  } catch {
    /* The generic assertion must not publish the issued URL. */
  }
  expect(valid, "Issued link must target the configured general web path").toBe(
    true,
  );
  await page.getByRole("button", { name: "Dismiss", exact: true }).click();
  await expect(field).toHaveCount(0);
  return value;
}

async function safeScreenshot(page: Page, name: string): Promise<void> {
  const url = new URL(page.url());
  const unsafe =
    url.searchParams.has("token") ||
    url.searchParams.has("proof") ||
    (await page.locator('input[type="password"]').count()) > 0 ||
    (await page
      .getByRole("textbox", { name: "New link", exact: true })
      .count()) > 0;
  expect(
    unsafe,
    "Screenshots require a credential-free workspace or dismissed inventory",
  ).toBe(false);
  const directory = fileURLToPath(new URL("../artifacts/", import.meta.url));
  await mkdir(directory, { recursive: true, mode: 0o700 });
  const render = await page.evaluate(async () => {
    await document.fonts.ready;
    const heading = document.querySelector("h1");
    const style = getComputedStyle(heading ?? document.body);
    return {
      viewport: { width: innerWidth, height: innerHeight },
      devicePixelRatio,
      locale: document.documentElement.lang,
      colorScheme: document.documentElement.dataset.mantineColorScheme,
      fontsReady: document.fonts.status === "loaded",
      loadedFamilies: [...document.fonts]
        .filter((font) => font.status === "loaded")
        .map((font) => font.family),
      heading: {
        family: style.fontFamily,
        size: style.fontSize,
        weight: style.fontWeight,
        lineHeight: style.lineHeight,
      },
      document: {
        width: document.documentElement.scrollWidth,
        height: document.documentElement.scrollHeight,
      },
    };
  });
  expect(render.fontsReady, "Product fonts must settle before capture").toBe(
    true,
  );
  expect(
    render.loadedFamilies.some((family) => family.includes("Pretendard")),
    "The actual product Pretendard font must be loaded",
  ).toBe(true);
  await page.screenshot({
    path: `${directory}${name}.png`,
    type: "png",
    fullPage: true,
  });
  await writeFile(
    `${directory}${name}.json`,
    JSON.stringify(render, null, 2) + "\n",
    { mode: 0o600 },
  );
}

async function confirm(page: Page): Promise<void> {
  const dialog = page.getByRole("dialog", {
    name: "Confirm this change",
    exact: true,
  });
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Confirm", exact: true }).click();
  await expect(dialog).toHaveCount(0);
}

test.describe("Assembled account and team acceptance", () => {
  test.describe.configure({ mode: "serial" });
  test.beforeAll(async ({ browser }) => {
    administrator = await isolatedPage(browser);
    member = await isolatedPage(browser);
    second = await isolatedPage(browser);
  });
  test.afterEach(() => {
    if (websiteErrors.length > 0) {
      throw new Error(
        `Website exception: ${websiteErrors.shift() ?? "Unexpected browser error"}`,
      );
    }
  });
  test.afterAll(async () => {
    let failed = false;
    for (const context of contexts) {
      try {
        await context.close();
      } catch {
        failed = true;
      }
    }
    if (failed) {
      throw new Error("Owned browser context cleanup failed");
    }
  });

  test("administrator bootstrap and issued signup reach the personal context", async ({
    assembly,
  }) => {
    const bootstrapRequests: string[] = [];
    administrator.on("request", (request) => {
      const operation = new URL(request.url()).pathname.split("/").at(-1);
      if (
        operation === "auth.methods" ||
        operation === "setup.status" ||
        operation === "auth.identity"
      )
        bootstrapRequests.push(operation);
    });
    await administrator.emulateMedia({ colorScheme: "dark" });
    await verifyInitialCanvas(
      administrator.context(),
      `${assembly.generalUrl}/login`,
      { serverMode: "light", mode: "dark", media: "dark" },
    );
    await verifyInitialCanvas(
      administrator.context(),
      `${assembly.adminUrl}/setup`,
      { serverMode: "dark", mode: "light", media: "light" },
    );
    await administrator.addInitScript(() => {
      function firstContentFrame(): void {
        if (document.documentElement.dataset.appearanceObservation === "done")
          return;
        if (!document.querySelector("main")) {
          requestAnimationFrame(firstContentFrame);
          return;
        }
        const root = document.documentElement;
        const mode = root.dataset.mantineColorScheme;
        const background = getComputedStyle(document.body).backgroundColor;
        root.dataset.firstFrameMode ??= mode;
        root.dataset.firstFrameBackground ??= background;
        if (background !== "rgb(24, 28, 24)")
          root.dataset.unexpectedFrameBackground = background;
        if (
          mode &&
          root.dataset.appearanceModeHistory?.split(",").at(-1) !== mode
        ) {
          root.dataset.appearanceModeHistory = root.dataset
            .appearanceModeHistory
            ? `${root.dataset.appearanceModeHistory},${mode}`
            : mode;
        }
        if (root.dataset.appearanceObservation !== "done")
          requestAnimationFrame(firstContentFrame);
      }
      requestAnimationFrame(firstContentFrame);
    });
    await administrator.goto(`${assembly.generalUrl}/login`);
    await expect(administrator.locator("html")).toHaveAttribute(
      "data-first-frame-mode",
      "dark",
    );
    await expect(administrator.locator("html")).toHaveAttribute(
      "data-first-frame-background",
      "rgb(24, 28, 24)",
    );
    const appearance = administrator.getByRole("button", {
      name: "Appearance",
      exact: true,
    });
    await expect(appearance).toHaveAttribute("data-mode", "system");
    const initialEmail = administrator.getByRole("textbox", {
      name: "Email address",
      exact: true,
    });
    await initialEmail.fill("draft@example.com");
    await administrator.evaluate(() => {
      document.documentElement.dataset.appearanceObservation = "done";
    });
    await expect(administrator.locator("html")).toHaveAttribute(
      "data-appearance-mode-history",
      "dark",
    );
    await expect(administrator.locator("html")).not.toHaveAttribute(
      "data-unexpected-frame-background",
    );
    await appearance.focus();
    await appearance.click();
    await administrator
      .getByRole("menuitem", { name: "Light", exact: true })
      .click();
    await expect(initialEmail).toHaveValue("draft@example.com");
    await expect(appearance).toBeFocused();
    await administrator.emulateMedia({ colorScheme: "light" });
    await appearance.click();
    await administrator
      .getByRole("menuitem", { name: "Dark", exact: true })
      .click();
    await expect(administrator.locator("body")).toHaveAttribute(
      "data-color-mode",
      "dark",
    );
    const rendered = await administrator.request.get(
      `${assembly.generalUrl}/login`,
    );
    expect(await rendered.text()).toMatch(
      /<html[^>]*data-mantine-color-scheme="dark"/,
    );
    await verifyInitialCanvas(
      administrator.context(),
      `${assembly.generalUrl}/login`,
    );
    await verifyInitialCanvas(
      administrator.context(),
      `${assembly.adminUrl}/setup`,
    );
    await administrator.reload();
    await expect(administrator.locator("html")).toHaveAttribute(
      "data-first-frame-background",
      "rgb(24, 28, 24)",
    );
    await expect(
      administrator.getByRole("button", { name: "Appearance", exact: true }),
    ).toHaveAttribute("data-mode", "dark");
    await administrator.evaluate(() => {
      document.documentElement.dataset.appearanceObservation = "done";
    });
    await expect(administrator.locator("html")).not.toHaveAttribute(
      "data-unexpected-frame-background",
    );
    await administrator.goto(`${assembly.adminUrl}/setup`);
    await expect(
      administrator.getByRole("button", { name: "Appearance", exact: true }),
    ).toHaveAttribute("data-mode", "dark");
    await expect(administrator.locator("html")).toHaveAttribute(
      "data-first-frame-mode",
      "dark",
    );
    await expect(administrator.locator("html")).toHaveAttribute(
      "data-first-frame-background",
      "rgb(24, 28, 24)",
    );
    await administrator
      .getByRole("textbox", { name: "Email", exact: true })
      .fill(assembly.administrator.email);
    await administrator
      .getByLabel(/^Setup proof\s*\*?$/)
      .fill(assembly.setupProof);
    await administrator
      .getByLabel(/^Password\s*\*?$/)
      .fill(assembly.administrator.password);
    expect(
      bootstrapRequests,
      "Initial hydration must not repeat server readiness reads",
    ).toEqual([]);
    await administrator
      .getByRole("button", { name: "Create administrator", exact: true })
      .click();
    await expect(
      administrator.getByRole("heading", { name: "Signup links", exact: true }),
    ).toBeVisible();
    expect(
      bootstrapRequests.filter((operation) => operation === "auth.identity"),
      "Protected hydration must not repeat server Identity validation",
    ).toEqual([]);
    await administrator
      .getByRole("textbox", { name: "Recipient email", exact: true })
      .fill(assembly.member.email);
    await administrator
      .getByRole("button", { name: "Create link", exact: true })
      .click();
    const signup = await issuedLink(administrator, assembly, "signup");
    await safeScreenshot(administrator, "admin-signup-inventory-desktop");
    await member.goto(signup);
    await member
      .getByRole("textbox", { name: "Email address", exact: true })
      .fill(assembly.member.email);
    await member.getByLabel(/^Password\s*\*?$/).fill(assembly.member.password);
    await member
      .getByRole("button", { name: "Save password", exact: true })
      .click();
    await expect(
      member.getByRole("heading", { name: "Your spaces", exact: true }),
    ).toBeVisible();
    await member
      .getByRole("main")
      .getByRole("link", { name: "Personal space", exact: true })
      .click();
    await expect(
      member.getByRole("heading", { name: "Personal space", exact: true }),
    ).toBeVisible();
    await expect(
      member.getByRole("button", { name: "Delete team", exact: true }),
    ).toHaveCount(0);
    await safeScreenshot(member, "personal-context-desktop");
    // Inspect attributes only on this test's isolated context; never assert values.
    const cookies = await member.context().cookies(assembly.generalUrl);
    const cookie = cookies.find(
      (item) => item.name === "chapchoo-general-session",
    );
    expect(Boolean(cookie), "General session cookie must be present").toBe(
      true,
    );
    expect(cookie?.httpOnly).toBe(true);
    expect(cookie?.secure).toBe(false);
    expect(cookie?.sameSite).toBe("Lax");
    expect(cookie?.path).toBe("/");
    expect(
      Boolean(
        cookie &&
        cookie.expires > Date.now() / 1000 &&
        cookie.expires <= Date.now() / 1000 + 30 * 86_400 + 60,
      ),
      "Cookie persistence is bounded",
    ).toBe(true);
  });

  test("registered invitation, ownership transfer and former-owner leave", async ({
    assembly,
  }) => {
    await assembly.seedSecondUser();
    await member.goto(`${assembly.generalUrl}/`);
    await member
      .getByRole("textbox", { name: "Team name", exact: true })
      .fill(assembly.teamName);
    await member
      .getByRole("textbox", { name: "Team handle", exact: true })
      .fill(assembly.teamHandle);
    await member
      .getByRole("button", { name: "Create team", exact: true })
      .click();
    await expect(
      member.getByRole("heading", { name: assembly.teamName, exact: true }),
    ).toBeVisible();
    const teamUrl = member.url();
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
    await second.getByRole("button", { name: "Accept", exact: true }).click();
    const teamLink = second
      .getByRole("main")
      .getByRole("link", { name: assembly.teamName, exact: true });
    await expect(teamLink).toBeVisible();
    await teamLink.click();
    await expect(
      second.getByRole("heading", { name: assembly.teamName, exact: true }),
    ).toBeVisible();
    // One representative ordinary-member affordance check, not a role matrix.
    await expect(
      second.getByRole("heading", { name: "Team settings", exact: true }),
    ).toHaveCount(0);
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
      member
        .getByRole("row")
        .filter({ hasText: assembly.member.email })
        .getByRole("cell", { name: "Manager", exact: true }),
    ).toBeVisible();
    await expect(
      target.getByRole("cell", { name: "Owner", exact: true }),
    ).toBeVisible();
    await member
      .getByRole("button", { name: "Leave team", exact: true })
      .click();
    await confirm(member);
    await expect(
      member.getByRole("heading", { name: "Your spaces", exact: true }),
    ).toBeVisible();
    await expect(
      member
        .getByRole("main")
        .getByRole("link", { name: assembly.teamName, exact: true }),
    ).toHaveCount(0);
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
    await safeScreenshot(second, "team-owner-desktop");
  });

  test("administrator reset revokes old sessions and ordinary users stay outside Admin", async ({
    assembly,
    browser,
  }) => {
    await administrator.goto(`${assembly.adminUrl}/password-resets`);
    await expect(
      administrator.getByRole("heading", {
        name: "Password reset links",
        exact: true,
      }),
    ).toBeVisible();
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
      (await resetPage.context().cookies(assembly.generalUrl)).some(
        (cookie) => cookie.name === "chapchoo-general-session",
      ),
      "Reset must not authenticate the recipient",
    ).toBe(false);
    await resetPage.goto(`${assembly.generalUrl}/`);
    await expect(resetPage).toHaveURL(`${assembly.generalUrl}/login`);
    await member.goto(`${assembly.generalUrl}/`);
    await expect(member).toHaveURL(`${assembly.generalUrl}/login`);
    await expect(
      member.getByRole("heading", { name: "Log in", exact: true }),
    ).toBeVisible();
    await login(member, assembly.generalUrl, {
      email: assembly.member.email,
      password: assembly.resetPassword,
    });
    await member
      .getByRole("main")
      .getByRole("link", { name: "Personal space", exact: true })
      .click();
    await expect(
      member.getByRole("heading", { name: "Personal space", exact: true }),
    ).toBeVisible();
    // The ordinary team owner retains General access but cannot enter Admin.
    await second.goto(`${assembly.adminUrl}/login`);
    await second
      .getByRole("textbox", { name: "Email", exact: true })
      .fill(assembly.second.email);
    await second.getByLabel(/^Password\s*\*?$/).fill(assembly.second.password);
    await second.getByRole("button", { name: "Sign in", exact: true }).click();
    // Exclude Next.js's separate route-announcer alert outside the form.
    await expect(second.getByRole("main").getByRole("alert")).toBeVisible();
    expect(
      (await second.context().cookies(assembly.adminUrl)).some(
        (cookie) => cookie.name === "chapchoo-admin-session",
      ),
      "Ordinary users must not receive an Admin cookie",
    ).toBe(false);
    await second.goto(`${assembly.adminUrl}/`);
    await expect(second).toHaveURL(`${assembly.adminUrl}/login`);
    await expect(
      second.getByRole("heading", { name: "Signup links", exact: true }),
    ).toHaveCount(0);
    await second.goto(`${assembly.generalUrl}/`);
    await expect(
      second.getByRole("heading", { name: "Your spaces", exact: true }),
    ).toBeVisible();
  });
});
