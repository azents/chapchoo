import { expect, test } from "@playwright/test";
import { instructionEvidenceCopy } from "../instructionEvidenceCopy";
const copy = instructionEvidenceCopy("en");
test("prompt sources are read-only originals with absent roles distinguished", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-prompt-sources--supplied&viewMode=story",
  );
  await expect(
    page.getByRole("combobox", { name: copy.execution, exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("status")).toHaveText(copy.supplied);
  await expect(
    page.getByText("한국어 원문도 그대로 유지합니다.", { exact: false }),
  ).toBeVisible();
  await expect(page.getByText(copy.absent, { exact: true })).toHaveCount(2);
  await expect(page.getByRole("textbox")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Save", exact: true }),
  ).toHaveCount(0);
});
test("prepared selection does not claim actual supply", async ({ page }) => {
  await page.goto(
    "/iframe.html?id=chat-prompt-sources--prepared&viewMode=story",
  );
  await expect(page.getByRole("status")).toHaveText(copy.prepared);
});
test("failed source reads and no retained evidence remain distinct", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-prompt-sources--source-read-failure&viewMode=story",
  );
  await expect(page.getByRole("alert")).toContainText(copy.error);
  await expect(
    page.getByRole("button", { name: copy.retry, exact: true }),
  ).toBeVisible();
  await expect(page.getByText(copy.absent, { exact: true })).toHaveCount(0);
  await page.goto("/iframe.html?id=chat-prompt-sources--empty&viewMode=story");
  await expect(page.getByText(copy.empty, { exact: true })).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
});
test("older loading and failures retain the original selected source", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-prompt-sources--older-loading&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: copy.more, exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByText("한국어 원문도 그대로 유지합니다.", { exact: false }),
  ).toBeVisible();
  await page.goto(
    "/iframe.html?id=chat-prompt-sources--older-read-failure&viewMode=story",
  );
  await expect(page.getByRole("alert")).toHaveText(copy.moreError);
  await expect(
    page.getByRole("button", { name: copy.more, exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByText("한국어 원문도 그대로 유지합니다.", { exact: false }),
  ).toBeVisible();
});
