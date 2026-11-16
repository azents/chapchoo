import { expect, test } from "@playwright/test";
import { instructionEvidenceCopy } from "../instructionEvidenceCopy";
const copy = instructionEvidenceCopy("en");
test("system prompt page renders exact current instructions and links back to the conversation", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=chat-system-prompt--applied&viewMode=story");
  await expect(
    page.getByRole("heading", { name: copy.title, exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("status")).toHaveText(copy.supplied);
  await expect(
    page.getByText("한국어 원문도 그대로 유지합니다.", { exact: false }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: copy.back })).toHaveAttribute(
    "href",
    "/teams/my/agents/companion/chat",
  );
});
test("prepared content and absent content remain distinct", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-system-prompt--prepared&viewMode=story",
  );
  await expect(page.getByRole("status")).toHaveText(copy.prepared);
  await page.goto("/iframe.html?id=chat-system-prompt--empty&viewMode=story");
  await expect(page.getByText(copy.empty, { exact: true })).toBeVisible();
});
test("Korean system prompt terminology is localized", async ({ page }) => {
  await page.goto("/iframe.html?id=chat-system-prompt--korean&viewMode=story");
  await expect(
    page.getByRole("heading", { name: "시스템 프롬프트", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "대화로 돌아가기" }),
  ).toBeVisible();
});
