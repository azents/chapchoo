import { expect, test } from "@playwright/test";

test("pending Markdown user bubbles are dimmed and have no lifecycle caption", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=chat-conversation-body--input-states&viewMode=story",
  );
  const messages = page
    .locator('[data-kind="user"]')
    .filter({ has: page.getByText("확인", { exact: true }) });
  await expect(messages).toHaveCount(2);
  const widths = await messages.evaluateAll((elements) =>
    elements.map(
      (element) => element.querySelector("div")?.getBoundingClientRect().width,
    ),
  );
  expect(widths[0]).toBe(widths[1]);
  await expect(messages.nth(0).locator("strong")).toHaveText("확인");
  await expect(messages.nth(1).locator("strong")).toHaveText("확인");
  for (const message of await messages.all()) {
    await expect(message).toHaveCSS("opacity", "0.6");
  }
  await expect(page.getByText("Delivery failed", { exact: true })).toHaveCount(
    0,
  );
  await expect(
    page.getByText("Delivery not confirmed", { exact: true }),
  ).toHaveCount(0);
});

test("permission choices start unselected and remain expanded after an explicit response", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--permission&viewMode=story",
  );
  const choices = page.getByRole("radio");
  await expect(choices).toHaveCount(3);
  for (const choice of await choices.all()) {
    await expect(choice).not.toBeChecked();
  }
  await expect(
    page.getByText("/workspace/current-project", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("cat ./notes/release.txt", { exact: false }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Send answers", exact: true }).click();
  await expect(page.getByRole("alert")).toHaveText("Check this answer");
  await page
    .getByRole("radio", {
      name: "Allow for this session Current session",
      exact: true,
    })
    .check();
  await page.getByRole("button", { name: "Send answers", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("Accepted");
  await expect(
    page.getByRole("radio", { name: /Allow for this session/ }),
  ).toBeChecked();
  await expect(choices).toHaveCount(3);
  await expect(page.getByText("Selected", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Send answers", exact: true }),
  ).toHaveCount(0);
});

test("form answers retain typed false, valid empty values, exact integers and optional non-answers", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--form&viewMode=story",
  );
  await expect(
    page.getByRole("textbox", { name: "Exact limit", exact: true }),
  ).toHaveValue("9223372036854775807");
  await page.getByRole("button", { name: "Send answers", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Required");
  await page
    .getByRole("radio", { name: "Public release", exact: true })
    .check();
  await page.getByRole("button", { name: "Send answers", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("Accepted");
  await expect(
    page.getByText("9223372036854775807", { exact: true }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("group", { name: /Enable notifications/ })
      .getByRole("radio", { name: /No/ }),
  ).toBeChecked();
  await expect(
    page
      .getByRole("group", { name: /Release note/ })
      .getByText("Empty answer", { exact: true }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("group", { name: /Targets/ })
      .getByText("Not answered", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("radio", { name: /Public release/ }),
  ).toBeChecked();
  await expect(
    page.getByRole("radio", { name: /Internal preview/ }),
  ).toBeVisible();
});

test("retained answers are fully readable without a live responder or a disclosure", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--retained&viewMode=story",
  );
  await expect(page.getByRole("status")).toHaveText("Response delivered");
  await expect(
    page.getByText("owner@example.com", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("9223372036854775807", { exact: true }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("group", { name: /Release channel/ })
      .getByRole("radio", { name: /Public release/ }),
  ).toBeChecked();
});

test("uncertain response delivery does not offer another effect", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--uncertain&viewMode=story",
  );
  await expect(page.getByRole("alert")).toHaveText("Delivery not confirmed");
  await expect(
    page.getByRole("button", { name: "Send answers", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Decline", exact: true }),
  ).toBeDisabled();
});

test("user and distinct assistant utterances render safe Markdown with activity separate", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--content&viewMode=story",
  );
  const user = page.locator('[data-kind="user"]');
  await expect(user.locator("strong")).toHaveText("User Markdown");
  await expect(user.getByRole("listitem")).toHaveCount(2);
  await expect(user.locator("code")).toHaveText("inline code");
  await expect(user.locator("script")).toHaveCount(0);
  await expect(user.getByRole("link")).toHaveAttribute(
    "rel",
    "noopener noreferrer",
  );
  await expect(user).toHaveAttribute("aria-label", "You");
  await expect(page.locator('[data-kind="agent"]')).toHaveCount(2);
  await expect(
    page.getByRole("heading", { name: "First utterance" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "a reference" })).toHaveAttribute(
    "rel",
    "noopener noreferrer",
  );
});

test("tool-only original input and output render as independent complete fields", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 844 });
  await page.goto(
    "/iframe.html?id=chat-conversation-body--history-content&viewMode=story",
  );
  await page
    .locator('[data-kind="tool"] button')
    .filter({ hasText: "Update notes" })
    .click();
  await page
    .locator('[data-kind="tool"] button')
    .filter({ hasText: "Read note" })
    .click();
  await page.locator('[data-kind="terminal"] button').first().click();
  await expect(
    page.getByText("− previous line", { exact: true }),
  ).toBeVisible();
  await expect(
    page
      .locator('[data-kind="terminal"] pre')
      .filter({ hasText: "Complete output" }),
  ).toBeVisible();
  await page.locator("summary").filter({ hasText: "Tool input" }).click();
  await expect(
    page.getByRole("region", { name: "Tool input", exact: true }),
  ).toContainText("9223372036854775807");
  await expect(
    page.getByRole("region", { name: "Tool output", exact: true }),
  ).toContainText("complete tool-only result");
  await expect(
    page.getByRole("region", { name: "Tool output", exact: true }),
  ).toContainText('"enabled":false');
  await expect
    .poll(() =>
      page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
    )
    .toBe(true);
});

test("a retained answer supersedes local uncertainty", async ({ page }) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--retained-after-uncertainty&viewMode=story",
  );
  await expect(page.getByRole("status")).toHaveText("Response delivered");
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(
    page.getByText("owner@example.com", { exact: true }),
  ).toBeVisible();
});

test("a definitive validation rejection keeps the original form editable for manual correction", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--validation-rejected&viewMode=story",
  );
  await expect(page.getByRole("alert")).toHaveText("Check this answer");
  const note = page.getByRole("textbox", { name: "Release note", exact: true });
  await expect(note).toBeEnabled();
  await note.fill("Corrected manually");
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Send answers", exact: true }),
  ).toBeEnabled();
});

test("history shows a captured user message without a second delivery row", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--captured-history&viewMode=story",
  );
  const captured = page.getByRole("button").filter({ hasText: "ㅎㅇ" });
  await expect(captured).toBeVisible();
  await expect(captured.getByText("나", { exact: true })).toHaveCount(0);
  await expect(page.getByText("메시지 · 에이전트에 전달됨")).toHaveCount(0);
  await expect(
    page.getByText("전달되지 않은 메시지", { exact: true }),
  ).toHaveCount(0);
  await expect(page.locator("[data-input-operation]")).toHaveCount(0);
});

for (const story of [
  "form",
  "permission",
  "retained",
  "korean",
  "japanese",
  "french",
]) {
  test(`expanded questions remain keyboard accessible without mobile overflow: ${story}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width: 320, height: 844 });
    await page.goto(
      `/iframe.html?id=chat-conversation-body--${story}&viewMode=story`,
    );
    await expect(page.getByRole("heading").first()).toBeVisible();
    await expect
      .poll(() =>
        page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
      )
      .toBe(true);
  });
}
