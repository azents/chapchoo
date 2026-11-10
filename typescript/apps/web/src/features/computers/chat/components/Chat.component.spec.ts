import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { expect, test } from "@playwright/test";

for (const width of [390, 1280]) {
  test(`activity exposes Agent details and a single no-plan history boundary at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.goto(
      "/iframe.html?id=chat-conversation--no-plan-activity&viewMode=story",
    );
    await page.getByRole("button", { name: /코기.*작업 상태/ }).click();
    const activity = page.getByRole("dialog", {
      name: "작업 상태",
      exact: true,
    });
    const details = activity.getByRole("link", {
      name: "에이전트 상세",
      exact: true,
    });
    await expect(details).toBeVisible();
    await expect(details).toHaveAttribute("href", "/teams/my/agents/corgi");
    await expect(activity.getByRole("heading", { level: 3 })).toHaveCount(0);
    const history = activity
      .locator("details")
      .filter({ hasText: "전체 기록" });
    await expect(history).toBeVisible();
    const boundaries = await history.evaluate((element) => {
      const summary = element.previousElementSibling;
      return {
        historyTop: getComputedStyle(element).borderTopStyle,
        summaryBottom: summary
          ? getComputedStyle(summary).borderBottomStyle
          : null,
        previousHeading: summary?.querySelector("h2")?.textContent,
      };
    });
    expect(boundaries).toEqual({
      historyTop: "solid",
      summaryBottom: "none",
      previousHeading: "에이전트 작업",
    });
    await expect
      .poll(() =>
        activity.evaluate(
          (element) => element.scrollWidth <= element.clientWidth,
        ),
      )
      .toBe(true);
    await details.focus();
    await expect(details).toBeFocused();
  });
}

for (const story of [
  "sending-with-active-work",
  "reader-error-with-active-work",
]) {
  test(`current-work Stop remains available independently of the composer/read state: ${story}`, async ({
    page,
  }) => {
    await page.goto(
      `/iframe.html?id=chat-conversation--${story}&viewMode=story`,
    );
    await expect(
      page.getByRole("textbox", { name: "Message", exact: true }),
    ).toBeDisabled();
    await page.getByRole("button", { name: /Corgi.*Activity/ }).click();
    const activity = page.getByRole("dialog", {
      name: "Activity",
      exact: true,
    });
    const stop = activity.getByRole("button", {
      name: "Stop work",
      exact: true,
    });
    await expect(stop).toBeEnabled();
    await stop.click();
    await expect(activity.getByRole("status")).toHaveText("Cancellation sent");
  });
}

test("a native text plan uses its typed text instead of empty generic content", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--text-plan&viewMode=story",
  );
  await page.getByRole("button", { name: /Corgi.*Activity/ }).click();
  await expect(
    page.getByRole("dialog", { name: "Activity", exact: true }),
  ).toContainText("First, inspect the current implementation.");
});

test("nested Computer dialogs retain activity and return focus on Escape", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=chat-conversation--english&viewMode=story");
  await page.getByRole("button", { name: /Corgi.*Activity/ }).click();
  const activity = page.getByRole("dialog", { name: "Activity", exact: true });
  const stop = activity.getByRole("button", { name: "Stop", exact: true });
  await stop.click();
  await expect(
    page.getByRole("dialog", { name: "Stop", exact: true }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(activity).toBeVisible();
  await expect(stop).toBeFocused();
  const details = activity.getByRole("button", {
    name: "Computer details",
    exact: true,
  });
  await details.click();
  await expect(
    page.getByRole("dialog", { name: "Computer details", exact: true }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(activity).toBeVisible();
  await expect(details).toBeFocused();
  await activity.getByRole("button", { name: "Close", exact: true }).click();
  await expect(
    page.getByRole("button", { name: /Corgi.*Activity/ }),
  ).toBeFocused();
});

test("failed Start is visible without opening activity", async ({ page }) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--failed-start&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: /코기.*작업 상태/ }),
  ).toContainText(computerCopy("ko-KR").lostSubmission);
});

test("measured input and overlay clearance honor enlarged root text", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=chat-conversation--english&viewMode=story");
  await page.evaluate(() => {
    document.documentElement.style.fontSize = "200%";
  });
  const input = page.getByRole("textbox", { name: "Message", exact: true });
  await input.fill("One\nTwo\nThree\nFour\nFive\nSix");
  const metrics = await input.evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      height: Number.parseFloat(style.height),
      line: Number.parseFloat(style.lineHeight),
    };
  });
  expect(metrics.height).toBeLessThanOrEqual(metrics.line * 5 + 1);
  expect(metrics.height).toBeGreaterThanOrEqual(metrics.line * 4);
  const log = page.getByRole("log", { name: "Message", exact: true });
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.clientHeight - element.scrollTop,
      ),
    )
    .toBeLessThan(2);
  await expect
    .poll(() =>
      page.evaluate(() => {
        const footer = document.querySelector("footer");
        if (!footer) {
          return false;
        }
        const actual = footer.getBoundingClientRect().height;
        const shell = footer.parentElement;
        return shell
          ? Math.abs(
              Number.parseFloat(
                getComputedStyle(shell).getPropertyValue("--composer-space"),
              ) - actual,
            ) < 1
          : false;
      }),
    )
    .toBe(true);
});

test("a wheel at the latest edge does not silently detach following", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=chat-conversation--english&viewMode=story");
  const log = page.getByRole("log", { name: "Message", exact: true });
  await log.hover();
  await page.mouse.wheel(0, 200);
  const input = page.getByRole("textbox", { name: "Message", exact: true });
  await input.fill("One\nTwo\nThree\nFour\nFive");
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.clientHeight - element.scrollTop,
      ),
    )
    .toBeLessThan(2);
  await page.mouse.move(800, 300);
  await page.mouse.wheel(0, -500);
  await expect(
    page.getByRole("button", { name: "Jump to latest", exact: true }),
  ).toBeVisible();
});

for (const width of [320, 390, 1280]) {
  test(`conversation input, activity and Agent navigation at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.goto(
      "/iframe.html?id=chat-conversation--connected&viewMode=story",
    );
    const main = page.getByRole("main", { name: "코기", exact: true });
    const input = page.getByRole("textbox", { name: "메시지", exact: true });
    await expect(input).toBeEnabled();
    await expect
      .poll(() =>
        page.evaluate(
          () => document.documentElement.scrollWidth <= window.innerWidth,
        ),
      )
      .toBe(true);
    await main.getByRole("button", { name: /코기.*작업 상태/ }).click();
    const activity = page.getByRole("dialog", {
      name: "작업 상태",
      exact: true,
    });
    await expect(activity).toBeVisible();
    await activity
      .getByRole("button", { name: "작업 중지", exact: true })
      .click();
    await expect(
      activity.getByText("취소 요청 전달됨", { exact: true }),
    ).toBeVisible();
    await activity.getByRole("button", { name: "닫기", exact: true }).click();
    await input.fill("첫 번째 메시지");
    await input.press("Enter");
    await expect(
      main.getByText("첫 번째 메시지", { exact: true }),
    ).toBeVisible();
    await input.fill("두 번째 메시지");
    await page.getByRole("button", { name: "보내기", exact: true }).click();
    await expect(
      main.getByText("두 번째 메시지", { exact: true }),
    ).toBeVisible();
    const log = page.getByRole("log", { name: "메시지", exact: true });
    await log.evaluate((element) => {
      element.scrollTop = 0;
    });
    await expect(
      page.getByRole("button", { name: "최신 대화로 이동", exact: true }),
    ).toBeVisible();
    await input.fill(
      "여러 줄\n입력해도\n이전 대화를\n읽는 위치를\n유지합니다\n여섯째 줄",
    );
    await expect
      .poll(() => log.evaluate((element) => element.scrollTop))
      .toBe(0);
    await page
      .getByRole("button", { name: "최신 대화로 이동", exact: true })
      .click();
    await expect
      .poll(() =>
        log.evaluate(
          (element) =>
            element.scrollHeight - element.clientHeight - element.scrollTop,
        ),
      )
      .toBeLessThan(2);
    await input.fill("");
    await main
      .getByRole("button", { name: "에이전트와 대화", exact: true })
      .click();
    const navigation = page.getByRole("dialog", {
      name: "에이전트와 대화",
      exact: true,
    });
    await expect(
      navigation.getByRole("link", { name: /비숑/ }),
    ).toHaveAttribute("href", "/teams/my/agents/bichon/chat");
    await expect(
      navigation.getByRole("link", { name: "새 에이전트", exact: true }),
    ).toHaveAttribute("href", "/teams/my/new-agent");
  });
}

test("unavailable observations keep history readable and input read-only", async ({
  page,
}) => {
  for (const state of [
    "stopped",
    "disconnected",
    "unknown",
    "stale",
    "stale-stopped",
  ]) {
    await page.goto(
      `/iframe.html?id=chat-conversation--${state}&viewMode=story`,
    );
    await expect(
      page.getByRole("textbox", { name: "메시지", exact: true }),
    ).toBeDisabled();
    await expect(
      page.getByText("I am checking the opening hours.", { exact: true }),
    ).toBeVisible();
    const main = page.getByRole("main");
    await expect(
      main.getByRole("button", {
        name: computerCopy("ko-KR").retry,
        exact: true,
      }),
    ).toHaveCount(0);
    await expect(
      main.locator("header").getByText("읽기 전용", { exact: true }),
    ).toHaveCount(0);
    await expect(
      main.getByRole("textbox", { name: "메시지", exact: true }),
    ).toHaveAttribute("placeholder", /읽기 전용/);
    await page.getByRole("button", { name: /코기.*작업 상태/ }).click();
    await expect(
      page.getByRole("dialog", { name: "작업 상태", exact: true }),
    ).toBeVisible();
  }
});

test("auth withdrawal replaces the protected conversation surface", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--signed-out&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: "로그인", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("log")).toHaveCount(0);
});

test("localized status and long identity remain usable on narrow screens", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 844 });
  for (const state of ["english", "japanese", "french"]) {
    await page.goto(
      `/iframe.html?id=chat-conversation--${state}&viewMode=story`,
    );
    await expect(page.getByRole("textbox")).toBeEnabled();
    await expect
      .poll(() =>
        page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
      )
      .toBe(true);
    await page.locator('button[aria-haspopup="dialog"]').click();
    await expect(page.getByRole("dialog")).toBeVisible();
  }
});
