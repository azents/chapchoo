import { expect, test } from "@playwright/test";
import messages from "../../../../messages/en-US.json" with { type: "json" };

test("login keeps its server-selected character during form re-renders", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=accounts-authentication--server-selected-character&viewMode=story",
  );
  const image = page.locator('img[src$="/05-husky-216.webp"]');
  await expect(image).toBeVisible();
  await page
    .getByRole("textbox", { name: messages.accounts.email })
    .fill("typing@example.invalid");
  await page.locator('input[type="password"]').fill("typing-password");
  await expect(
    page.getByRole("textbox", { name: messages.accounts.email }),
  ).toHaveValue("typing@example.invalid");
  await expect(image).toBeVisible();
  await expect(page.locator('img[src*="/design-assets/"]')).toHaveCount(1);
});

test("supplementary invitation failure preserves primary contexts and actions", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=accounts-workspace--supplementary-unavailable&viewMode=story",
  );
  await expect(
    page.getByRole("heading", { name: "Your spaces", exact: true }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("main")
      .getByRole("link", { name: "Personal space", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Create team", exact: true }),
  ).toBeVisible();
});

test("reset completion component announces the app-owned exact message", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=accounts-authentication--reset-complete&viewMode=story",
  );
  await expect(page.getByRole("status")).toHaveText(
    messages.accounts.resetDone,
  );
  await expect(
    page.getByRole("link", { name: messages.accounts.backToLogin }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: messages.accounts.savePassword }),
  ).toHaveCount(0);
});

test("owner workspace component fits a 375-wide viewport", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/iframe.html?id=accounts-workspace--owner&viewMode=story");
  await expect(
    page.getByRole("heading", { name: "Design team", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
});

test("selected web typography and control surfaces are applied on narrow auth", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 850 });
  await page.goto(
    "/iframe.html?id=accounts-authentication--password-login&viewMode=story",
  );
  const email = page.getByRole("textbox", { name: messages.accounts.email });
  await expect(email).toHaveCSS("background-color", "rgb(255, 254, 250)");
  await expect(email).toHaveCSS("border-color", "rgb(112, 119, 101)");
  await expect(email).toHaveCSS("min-height", "44px");
  await expect(email).toHaveCSS("font-size", "16px");
  await expect(page.locator('input[type="password"]')).toHaveCSS(
    "font-size",
    "16px",
  );
  await expect(page.locator("body")).toHaveCSS(
    "font-family",
    "Pretendard, sans-serif",
  );
  await expect(
    page.getByRole("button", { name: messages.accounts.showPassword }),
  ).toHaveCSS("min-height", "44px");
  const fonts = await page.evaluate(async () => {
    await document.fonts.ready;
    return {
      loaded: [...document.fonts]
        .filter((font) => font.status === "loaded")
        .map((font) => font.family),
      resources: performance
        .getEntriesByType("resource")
        .filter(
          (entry): entry is PerformanceResourceTiming =>
            entry instanceof PerformanceResourceTiming &&
            entry.name.includes("/fonts/") &&
            entry.name.endsWith(".woff2"),
        )
        .map((entry) => ({ name: entry.name, bytes: entry.encodedBodySize })),
    };
  });
  expect(fonts.loaded).toContain("Pretendard");
  expect(fonts.resources.length).toBeGreaterThan(0);
  expect(
    fonts.resources.every((font) => font.name.includes("/pretendard/")),
  ).toBe(true);
  expect(
    fonts.resources.reduce((total, font) => total + font.bytes, 0),
  ).toBeLessThan(200_000);
});

test("narrow text-editing controls share the minimum size without restricting zoom", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await page.goto(
    "/iframe.html?id=shared-ui-fields--editable-controls&viewMode=story",
  );
  const editors = page.locator(
    'input:not([type="hidden"]),textarea,select,[contenteditable="true"],[contenteditable=""],[contenteditable="plaintext-only"]',
  );
  await expect(page.getByRole("searchbox", { name: "Search" })).toBeVisible();
  expect(await editors.count()).toBeGreaterThanOrEqual(13);
  for (const editor of await editors.all()) {
    await expect(editor).toHaveCSS("font-size", "16px");
    // Native selects retain their platform-owned line box.
    if (await editor.evaluate((element) => element.tagName !== "SELECT")) {
      await expect(editor).toHaveCSS("line-height", "20px");
    }
  }
  const viewport = await page
    .locator('meta[name="viewport"]')
    .getAttribute("content");
  expect(viewport).not.toMatch(/user-scalable=no|maximum-scale=1(?:,|$)/);
  await page.evaluate(() => {
    document.documentElement.style.fontSize = "200%";
  });
  await expect(page.getByRole("searchbox", { name: "Search" })).toHaveCSS(
    "font-size",
    "32px",
  );
});

test("shared action labels retain complete line boxes when wrapped or enlarged", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 950 });
  await page.goto(
    "/iframe.html?id=shared-ui-control-typography--labels&viewMode=story",
  );
  const labels = page.locator(".mantine-Button-label,.mantine-Chip-label");
  await expect(labels).toHaveCount(6);
  for (const size of ["100%", "200%"]) {
    await page.evaluate((value) => {
      document.documentElement.style.fontSize = value;
    }, size);
    for (const label of await labels.all()) {
      await expect(label).toHaveCSS("text-box-trim", "none");
      const metrics = await label.evaluate((element) => {
        const style = getComputedStyle(element);
        const root = element.closest("button") ?? element;
        const box = root.getBoundingClientRect();
        const text = document.createRange();
        text.selectNodeContents(element);
        return {
          lineHeight: parseFloat(style.lineHeight),
          fontSize: parseFloat(style.fontSize),
          loading: root.hasAttribute("data-loading"),
          textFits: [...text.getClientRects()].every(
            (rect) => rect.top >= box.top && rect.bottom <= box.bottom,
          ),
        };
      });
      expect(metrics.lineHeight / metrics.fontSize).toBeCloseTo(1.5);
      // Mantine intentionally translates the pending label behind its spinner.
      if (!metrics.loading) {
        expect(metrics.textFits).toBe(true);
      }
    }
  }
});

test("auth footer links lead to other modes rather than the current screen", async ({
  page,
}) => {
  for (const [story, current] of [
    ["password-login", "/login"],
    ["request-signup", "/signup"],
    ["administrator-link-required", "/reset-password"],
  ]) {
    await page.goto(
      `/iframe.html?id=accounts-authentication--${story}&viewMode=story`,
    );
    await expect(page.locator("main")).toBeVisible();
    await expect(page.locator(`a[href="${current}"]`)).toHaveCount(0);
    await expect(page.locator("main a")).toHaveCount(2);
  }
});
