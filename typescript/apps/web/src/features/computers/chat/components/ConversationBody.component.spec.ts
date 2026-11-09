import { expect, test } from "@playwright/test";

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

test("user text stays plain while distinct assistant utterances render Markdown and activity remains separate", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation-body--content&viewMode=story",
  );
  const user = page.locator('[data-kind="user"]');
  await expect(user).toContainText(
    "**This is plain user text**, not Markdown.",
  );
  await expect(user.locator("strong")).toHaveCount(1);
  await expect(page.locator('[data-kind="assistant"]')).toHaveCount(2);
  await expect(
    page.getByRole("heading", { name: "First utterance" }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "a reference" })).toHaveAttribute(
    "rel",
    "noopener noreferrer",
  );
  await page.locator('[data-kind="tool"] summary').click();
  await expect(page.getByText("previous line", { exact: true })).toBeVisible();
  await page.locator('[data-kind="terminal"] summary').first().click();
  await expect(
    page.getByText("Complete output", { exact: true }),
  ).toBeVisible();
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
