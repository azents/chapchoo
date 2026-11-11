import { expect, test } from "@playwright/test";

test("underfilled history loads successive pages without losing bottom follow", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-full-history--underfilled-pagination&viewMode=story",
  );
  const log = page.getByRole("log", { name: "Full history", exact: true });
  await expect(page.locator('[id="history-assistant%3A20"]')).toBeAttached();
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.scrollTop - element.clientHeight,
      ),
    )
    .toBeLessThan(2);
  await page
    .getByRole("button", { name: "Append remote record", exact: true })
    .click();
  await expect(page.locator('[id="history-assistant%3A41"]')).toBeAttached();
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.scrollTop - element.clientHeight,
      ),
    )
    .toBeLessThan(2);
});

test("history follows appended records while latest is selected", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=chat-full-history--following-latest&viewMode=story",
  );
  const log = page.getByRole("log", { name: "Full history", exact: true });
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.scrollTop - element.clientHeight,
      ),
    )
    .toBeLessThan(2);
  await page
    .getByRole("button", { name: "Append remote record", exact: true })
    .click();
  await expect(page.locator('[id="history-assistant%3A41"]')).toBeVisible();
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.scrollTop - element.clientHeight,
      ),
    )
    .toBeLessThan(2);
});

test("selected history entry expands and older pagination preserves the reading anchor", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-full-history--focused-record&viewMode=story",
  );
  const log = page.getByRole("log", { name: "Full history", exact: true });
  const selected = page.locator('[id="history-assistant%3A30"]');
  await expect(selected.locator("button").first()).toHaveAttribute(
    "aria-expanded",
    "true",
  );
  await expect(selected).toBeFocused();
  const before = await log.evaluate((element) => {
    element.scrollTop = 0;
    return (
      element
        .querySelector('[id="history-assistant%3A11"]')
        ?.getBoundingClientRect().top ?? 0
    );
  });
  const anchor = page.locator('[id="history-assistant%3A11"]');
  await expect(page.locator('[id="history-assistant%3A1"]')).toBeAttached();
  await expect
    .poll(() =>
      anchor.evaluate((element) => element.getBoundingClientRect().top),
    )
    .toBeCloseTo(before, 0);
  await page
    .getByRole("button", { name: "Update existing record", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "New events below", exact: true }),
  ).toHaveCount(0);
  await page
    .getByRole("button", { name: "Append remote record", exact: true })
    .click();
  await expect
    .poll(() =>
      anchor.evaluate((element) => element.getBoundingClientRect().top),
    )
    .toBeCloseTo(before, 0);
  await page
    .getByRole("button", { name: "New events below", exact: true })
    .click();
  await expect
    .poll(() =>
      log.evaluate(
        (element) =>
          element.scrollHeight - element.scrollTop - element.clientHeight,
      ),
    )
    .toBeLessThan(2);
});
