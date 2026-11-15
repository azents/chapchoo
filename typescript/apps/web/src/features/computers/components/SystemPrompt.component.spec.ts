import { expect, test } from "@playwright/test";
import { systemPromptCopy } from "../systemPromptCopy";

const copy = systemPromptCopy("en");
test("system prompt uses Save-only editing, preserves failed text and allows clearing", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-system-prompt--interactive-editor&viewMode=story",
  );
  const open = page.getByRole("button", { name: copy.edit, exact: true });
  await open.click();
  const field = page.getByRole("textbox", { name: copy.field, exact: true });
  await expect(field).toHaveValue("Original instructions");
  await expect(
    page.getByRole("button", { name: "Cancel", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Apply", exact: true }),
  ).toHaveCount(0);
  await field.fill("fail");
  await page.getByRole("button", { name: copy.save, exact: true }).click();
  await expect(page.getByText(copy.saveFailed, { exact: true })).toBeVisible();
  await expect(field).toHaveValue("fail");
  await page.getByRole("button", { name: copy.close, exact: true }).click();
  await expect(open).toBeFocused();
  await open.click();
  await expect(field).toHaveValue("Original instructions");
  await field.fill("");
  await page.getByRole("button", { name: copy.save, exact: true }).click();
  await expect(field).toHaveCount(0);
  await open.click();
  await expect(field).toHaveValue("");
});
test("failed reading never offers an empty editable setting", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-system-prompt--read-failure&viewMode=story",
  );
  await expect(page.getByRole("alert")).toHaveText(copy.readFailed);
  await expect(page.getByRole("textbox")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: copy.save, exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: copy.retry, exact: true }),
  ).toBeVisible();
});
test("pending save prevents duplicate edits and submission", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-system-prompt--saving&viewMode=story",
  );
  await expect(
    page.getByRole("textbox", { name: copy.field, exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: copy.save, exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: copy.close, exact: true }),
  ).toBeDisabled();
});
test("desktop editor is centered and mobile editor fills the viewport without horizontal overflow", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-system-prompt--editing&viewMode=story",
  );
  const dialog = page.getByRole("dialog");
  const desktop = await dialog.boundingBox();
  expect(desktop).not.toBeNull();
  expect(desktop?.width).toBeLessThan(1000);
  expect(desktop?.x).toBeGreaterThan(0);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=computers-system-prompt--mobile&viewMode=story",
  );
  const ko = systemPromptCopy("ko");
  await expect(
    page.getByRole("textbox", { name: ko.field, exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: ko.save, exact: true }),
  ).toBeVisible();
  const mobile = await dialog.boundingBox();
  expect(mobile?.width).toBe(390);
  expect(mobile?.height).toBe(844);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(390);
});
