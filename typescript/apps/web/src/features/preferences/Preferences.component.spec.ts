import { displayChoices } from "@chapchoo/web-core/display";
import { expect, test } from "@playwright/test";
import superjson from "superjson";
import type { DisplayValue } from "@chapchoo/web-core/display";
import type { Page } from "@playwright/test";

async function settingsTransport(
  page: Page,
  failFirst = false,
): Promise<() => number> {
  let calls = 0;
  let display: DisplayValue = {
    language: "ko",
    timeZone: "Asia/Seoul",
    region: "ko-KR",
  };
  await page.route("**/api/trpc/auth.updateDisplay**", async (route) => {
    expect(route.request().method()).toBe("POST");
    calls++;
    if (failFirst && calls === 1) {
      await route.fulfill({
        status: 503,
        json: {
          error: {
            json: {
              message: "Unavailable",
              code: -32603,
              data: {
                code: "INTERNAL_SERVER_ERROR",
                path: "auth.updateDisplay",
                reason: null,
                httpStatus: 503,
              },
            },
          },
        },
      });
      return;
    }
    const body: unknown = route.request().postDataJSON();
    if (!body || typeof body !== "object" || !("json" in body)) {
      throw new Error("Expected typed preference input");
    }
    display = { ...display, ...displayChoices(body.json) };
    await route.fulfill({
      json: {
        result: {
          data: superjson.serialize({
            id: "fixture-user",
            email: "owner@example.com",
            isAdmin: false,
            createdAt: { seconds: 0n, nanos: 0 },
            display,
          }),
        },
      },
    });
  });
  return (): number => calls;
}
const story = (scene: string): string =>
  `/iframe.html?id=preferences-integrated--${scene}&viewMode=story`;
async function select(
  page: Page,
  scope: "dialog" | "main",
  name: string,
  value: string,
): Promise<void> {
  await page
    .getByRole(scope)
    .getByRole("combobox", { name, exact: true })
    .click();
  await page.getByRole("option", { name: value, exact: true }).click();
}

test("production panel and profile edit the same committed identity snapshot in both directions", async ({
  page,
}) => {
  const calls = await settingsTransport(page);
  await page.goto(story("account"));
  await page
    .getByRole("button", { name: "언어 및 지역 설정", exact: true })
    .click();
  await select(page, "dialog", "시간대", "America/New_York");
  await expect(
    page.getByRole("dialog").getByText("반영됨", { exact: true }),
  ).toBeVisible();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "완료", exact: true })
    .click();
  await page.getByRole("button", { name: "프로필 보기", exact: true }).click();
  await expect(
    page
      .getByRole("main")
      .getByRole("combobox", { name: "시간대", exact: true }),
  ).toHaveValue("America/New_York");
  await select(page, "main", "지역 형식", "France · Français");
  await page
    .getByRole("button", { name: "언어 및 지역 설정", exact: true })
    .click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "지역 형식", exact: true }),
  ).toHaveValue("France · Français");
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "언어", exact: true }),
  ).toHaveValue("한국어");
  expect(calls()).toBe(2);
});

test("titled icon suggestions apply independently and dismissal survives reload until detected values change", async ({
  page,
}) => {
  const calls = await settingsTransport(page);
  await page.goto(story("profile"));
  const toast = page
    .getByRole("status")
    .filter({ hasText: "표시 설정을 바꿀까요?" });
  await expect(toast.locator("[data-proposal]")).toHaveCount(3);
  await expect(toast.locator("[data-proposal] svg")).toHaveCount(3);
  await toast.locator('[data-proposal="language"]').getByRole("button").click();
  await expect(
    page
      .getByRole("main")
      .getByRole("combobox", { name: "Language", exact: true }),
  ).toHaveValue("English");
  await expect(
    page
      .getByRole("main")
      .getByRole("combobox", { name: "Regional format", exact: true }),
  ).toHaveValue("대한민국 · 한국어");
  const remaining = page.locator("[data-proposal]");
  await expect(remaining).toHaveCount(2);
  await page
    .getByRole("button", { name: "Dismiss suggestions", exact: true })
    .click();
  await expect(remaining).toHaveCount(0);
  expect(calls()).toBe(1);
  await page.reload();
  await expect(
    page.getByRole("main").getByRole("combobox", { name: "언어", exact: true }),
  ).toHaveValue("한국어");
  await expect(page.locator('[data-proposal="timeZone"]')).toHaveCount(0);
  await expect(page.locator('[data-proposal="region"]')).toHaveCount(0);
  await page.addInitScript(() =>
    Object.defineProperty(navigator, "languages", { get: () => ["ja-JP"] }),
  );
  await page.reload();
  await expect(page.locator('[data-proposal="language"]')).toContainText(
    "日本語",
  );
  await expect(page.locator('[data-proposal="region"]')).toHaveCount(0);
  await expect(page.locator('[data-proposal="timeZone"]')).toHaveCount(0);
});

test("visitor explicit choice persists and browser following can be restored without a login form", async ({
  page,
}) => {
  await page.goto(story("guest"));
  await page
    .getByRole("button", { name: "Language and region settings", exact: true })
    .click();
  await select(page, "dialog", "Time zone", "Europe/London");
  await page.reload();
  await page
    .getByRole("button", { name: "Language and region settings", exact: true })
    .click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "Time zone", exact: true }),
  ).toHaveValue("Europe/London");
  await select(page, "dialog", "Time zone", "Follow browser settings");
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "Time zone", exact: true }),
  ).toHaveValue("Follow browser settings");
});

test("unconfirmed save retains selection and only an explicit retry submits again", async ({
  page,
}) => {
  const calls = await settingsTransport(page, true);
  await page.goto(story("profile"));
  await page
    .getByRole("button", { name: "언어 및 지역 설정", exact: true })
    .click();
  await select(page, "dialog", "시간대", "America/New_York");
  await expect(page.getByRole("dialog").getByRole("alert")).toBeVisible();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("combobox", { name: "시간대", exact: true }),
  ).toHaveValue("America/New_York");
  expect(calls()).toBe(1);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "다시 시도", exact: true })
    .click();
  await expect(page.getByRole("dialog").getByRole("alert")).toHaveCount(0);
  expect(calls()).toBe(2);
});

for (const width of [1280, 390, 320]) {
  test(`global settings and suggestion controls reflow and dismiss at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(story("profile"));
    await expect(page.locator("[data-proposal]")).toHaveCount(3);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth > innerWidth,
    );
    expect(overflow).toBe(false);
    await page
      .getByRole("button", { name: "변경 제안 닫기", exact: true })
      .click();
    const entry = page.getByRole("button", {
      name: "언어 및 지역 설정",
      exact: true,
    });
    await entry.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect(
      page
        .getByRole("dialog")
        .getByRole("button", { name: "설정 닫기", exact: true }),
    ).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(entry).toBeFocused();
  });
}
