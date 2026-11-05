import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { expect, test } from "@playwright/test";
import { computerDetailCopy } from "../detailCopy";
const c = computerCopy("en-US");
const d = computerDetailCopy("en-US");

test("an unavailable Restart keeps its independent reason reachable by keyboard", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-computer-controls--restart-unsupported&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: c.stop, exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByRole("button", { name: c.restart, exact: true }),
  ).toBeDisabled();
  await page.getByRole("group", { name: c.restart, exact: true }).focus();
  await expect(page.getByRole("tooltip")).toHaveText(
    `${c.restart}: ${c.unsupported}`,
  );
});

test("one primary power action follows the current power and server admission", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-computer-controls--on&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: c.stop, exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByRole("button", { name: c.start, exact: true }),
  ).toHaveCount(0);
  await expect(page.getByText(c.requiresStopped, { exact: true })).toHaveCount(
    0,
  );
  await page.goto(
    "/iframe.html?id=computers-computer-controls--off&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: c.start, exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByRole("button", { name: c.stop, exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: c.restart, exact: true }),
  ).toBeDisabled();
  for (const state of ["pending", "stale"]) {
    await page.goto(
      `/iframe.html?id=computers-computer-controls--${state}&viewMode=story`,
    );
    await expect(
      page.getByRole("button", { name: c.stop, exact: true }),
    ).toBeDisabled();
    await expect(
      page.getByRole("button", { name: c.restart, exact: true }),
    ).toBeDisabled();
  }
});

test("details owns the independent facts, optional diagnostics and Escape focus return", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-computer-controls--on&viewMode=story",
  );
  const trigger = page.getByRole("button", { name: c.details, exact: true });
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: c.details, exact: true });
  await expect(
    dialog.getByRole("heading", { name: d.on, exact: true }),
  ).toBeVisible();
  for (const label of [d.connection, d.management, d.storage, d.operation]) {
    await expect(dialog.getByText(label, { exact: true })).toBeVisible();
  }
  await expect(
    dialog.getByText(`${c.intent}: ${c.running}`, { exact: true }),
  ).toBeHidden();
  await dialog.getByText(d.diagnostics, { exact: true }).click();
  await expect(
    dialog.getByText(`${c.intent}: ${c.running}`, { exact: true }),
  ).toBeVisible();
  await dialog.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(trigger).toBeFocused();
});
