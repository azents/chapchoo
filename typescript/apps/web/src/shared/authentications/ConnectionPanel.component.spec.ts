import { authenticationCopy } from "@chapchoo/web-core/authentication-messages";
import { expect, test } from "@playwright/test";
const copy = authenticationCopy("ko");
test("device code has a copy action and successful feedback", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto(
    "/iframe.html?id=authentication-connections--device-code&viewMode=story",
  );
  await expect(page.getByText("ABCD-EFGH", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: copy.copyCode, exact: true }).click();
  await expect(
    page.getByRole("button", { name: copy.copied, exact: true }),
  ).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
    "ABCD-EFGH",
  );
});
test("empty alias remains optional and creation time identifies unnamed connections", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-connections--without-alias&viewMode=story",
  );
  const time = page.locator("time");
  await expect(time).toHaveAttribute("dateTime", "2026-11-06T00:00:00.000Z");
  const heading = page.getByRole("heading", { level: 3 });
  await expect(heading).not.toHaveText("");
  await expect(heading).toHaveText(await time.innerText());
  await expect(time).toHaveText(
    new Intl.DateTimeFormat("ko-KR", {
      dateStyle: "medium",
      timeStyle: "short",
      timeZone: "Asia/Seoul",
    }).format(Date.parse("2026-11-06T00:00:00Z")),
  );
  await page.goto(
    "/iframe.html?id=authentication-connections--optional-alias&viewMode=story",
  );
  const alias = page.getByRole("textbox", { name: copy.name, exact: true });
  await expect(alias).toBeEmpty();
  await expect(alias).not.toHaveAttribute("required");
  await expect(
    page.getByRole("button", { name: copy.save, exact: true }),
  ).toBeEnabled();
});
test("forward cursor pages label their reset action as First page", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-connections--later-page&viewMode=story",
  );
  await page.getByRole("button", { name: copy.first, exact: true }).click();
  await expect(
    page.getByRole("button", { name: copy.first, exact: true }),
  ).toHaveCount(0);
});

test("catalogue keeps record identity and bounded API storage meaning distinct", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-connections--populated&viewMode=story",
  );
  await expect(
    page.getByRole("heading", { name: "건우의 ChatGPT" }),
  ).toBeVisible();
  await expect(
    page.getByText("gunwoo@example.com", { exact: true }),
  ).toBeVisible();
  await expect(page.getByText(copy.unverified, { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: copy.manage })).toHaveCount(2);
  await page.goto(
    "/iframe.html?id=authentication-connections--read-only-team&viewMode=story",
  );
  await expect(page.getByRole("button", { name: copy.manage })).toHaveCount(0);
  await expect(page.getByText(copy.ownerOnly)).toBeVisible();
});
test("in-place key entry is masked with official issuance and separate billing before submission", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-connections--key-entry&viewMode=story",
  );
  const dialog = page.getByRole("dialog", { name: copy.apiKey, exact: true });
  await expect(dialog.getByLabel(/^API 키\s*\*?$/)).toHaveAttribute(
    "type",
    "password",
  );
  await expect(
    dialog.getByRole("link", { name: copy.issueKey }),
  ).toHaveAttribute("href", "https://platform.openai.com/api-keys");
  await expect(dialog.getByText(copy.billing)).toBeVisible();
  await expect(
    dialog.getByRole("button", { name: copy.save, exact: true }),
  ).toBeDisabled();
  await dialog.getByRole("button", { name: copy.cancel, exact: true }).click();
  await expect(dialog).toHaveCount(0);
});
test("device interaction displays the actual code, official destination, validity and cancellation", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-connections--device-code&viewMode=story",
  );
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByText("ABCD-EFGH", { exact: true })).toBeVisible();
  await expect(
    dialog.getByRole("link", { name: copy.openDevice }),
  ).toHaveAttribute("href", "https://auth.openai.com/codex/device");
  await expect(
    dialog.getByText(`${copy.remaining} 4:00`, { exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByRole("button", { name: copy.copyCode, exact: true }),
  ).toBeEnabled();
  await expect(
    dialog.getByRole("button", { name: copy.cancel, exact: true }),
  ).toBeEnabled();
});
test("uncertain mutation keeps recovery as the only action and unavailable reads are not empty catalogues", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=authentication-connections--uncertain-request&viewMode=story",
  );
  await expect(
    page
      .getByRole("dialog")
      .getByRole("button", { name: copy.recover, exact: true }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.goto(
    "/iframe.html?id=authentication-connections--unavailable&viewMode=story",
  );
  await expect(page.getByText(copy.unavailable, { exact: true })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: copy.emptyTitle }),
  ).toHaveCount(0);
});
test("key and device dialogs reflow without hiding their actions on narrow screens", async ({
  page,
}) => {
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    for (const story of ["key-entry", "device-code", "picker"]) {
      await page.goto(
        `/iframe.html?id=authentication-connections--${story}&viewMode=story`,
      );
      await expect(
        page.getByRole("button", {
          name:
            story === "key-entry"
              ? copy.save
              : story === "device-code"
                ? copy.copyCode
                : copy.selected,
          exact: true,
        }),
      ).toBeVisible();
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      ).toBe(true);
    }
  }
});
