import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { expect, test } from "@playwright/test";
const c = computerCopy("en-US");
test.use({ hasTouch: true });

test("blocked shutdown does not promise that startup can proceed", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-computer-controls--blocked-shutdown&viewMode=story",
  );
  await expect(page.getByText(`· ${c.blocked}`, { exact: true })).toBeVisible();
  await expect(page.getByText(c.blockedHint, { exact: true })).toBeVisible();
  await expect(
    page.getByRole("button", { name: c.startAfterStop, exact: true }),
  ).toHaveCount(0);
  await expect(page.getByText(c.startAfterStop, { exact: true })).toHaveCount(
    0,
  );
  await expect(
    page.getByRole("button", { name: c.stop, exact: true }),
  ).toBeDisabled();
});

test("deletion consent describes the removed data rather than cleanup ordering", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agents--power-confirmation-interaction-fixture&viewMode=story",
  );
  await page.getByRole("button", { name: c.delete, exact: true }).click();
  const dialog = page.getByRole("dialog", { name: c.delete, exact: true });
  await expect(
    dialog.getByText(c.deleteWarning, { exact: true }),
  ).toBeVisible();
  await expect(page.getByLabel("Submitted power operation")).toHaveText(
    "No operation submitted",
  );
  await dialog.getByRole("button", { name: c.cancel, exact: true }).click();
  await expect(page.getByLabel("Submitted power operation")).toHaveText(
    "No operation submitted",
  );
});

test("shutdown progress permits server-admitted Start without claiming stopped power", async ({
  page,
}) => {
  for (const state of ["stopping", "verifying-shutdown"]) {
    await page.goto(
      `/iframe.html?id=computers-computer-controls--${state}&viewMode=story`,
    );
    await expect(
      page.getByRole("button", { name: c.start, exact: true }),
    ).toBeEnabled();
    await expect(
      page.getByRole("button", { name: c.stop, exact: true }),
    ).toHaveCount(0);
    await expect(
      page.getByText(`Computer · ${c.stopped}`, { exact: true }),
    ).toHaveCount(0);
    await expect(page.getByText(c.uncertain, { exact: true })).toHaveCount(0);
  }
  await expect(
    page.getByText(`· ${c.checkingStop}`, { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText(c.checkingStopHint, { exact: true }),
  ).toBeVisible();
  await page.goto(
    "/iframe.html?id=computers-computer-controls--start-after-shutdown&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: c.startAfterStop, exact: true }),
  ).toBeDisabled();
  await expect(page.getByText(c.conflicting, { exact: true })).toHaveCount(0);
});

test("Stop and Restart require explicit consent on desktop and mobile", async ({
  page,
}) => {
  for (const width of [1280, 390]) {
    await page.setViewportSize({ width, height: 844 });
    for (const action of [c.stop, c.restart]) {
      await page.goto(
        "/iframe.html?id=computers-agents--power-confirmation-interaction-fixture&viewMode=story",
      );
      const submitted = page.getByLabel("Submitted power operation");
      const trigger = page.getByRole("button", { name: action, exact: true });
      if (width === 390) {
        await trigger.tap();
      } else {
        await trigger.click();
      }
      const dialog = page.getByRole("dialog", { name: action, exact: true });
      await expect(dialog).toBeVisible();
      await expect(submitted).toHaveText("No operation submitted");
      await dialog.getByRole("button", { name: c.cancel, exact: true }).click();
      await expect(dialog).toHaveCount(0);
      await expect(submitted).toHaveText("No operation submitted");
      await page.getByRole("button", { name: action, exact: true }).focus();
      await page.keyboard.press("Enter");
      await expect(dialog).toBeVisible();
      await expect(submitted).toHaveText("No operation submitted");
      await dialog
        .getByRole("button", { name: c.confirm, exact: true })
        .click();
      await expect(submitted).toHaveText(
        `${action === c.stop ? "stop" : "restart"}:observed-1`,
      );
    }
  }
});
