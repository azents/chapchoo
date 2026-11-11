import { expect, test } from "@playwright/test";

test("raw tool chunks join only adjacent text and preserve typed-content order", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-ordered-content--mixed-tool-output&viewMode=story",
  );
  await expect(page.locator("#storybook-root")).toContainText("alphabeta");
  await expect(page.locator("#storybook-root > pre")).toHaveText([
    "alphabeta",
    "gamma",
    "delta",
  ]);
  const order = await page
    .locator("#storybook-root")
    .evaluate((element) => element.textContent);
  expect(order.indexOf("alphabeta")).toBeLessThan(
    order.indexOf("Resource boundary"),
  );
  expect(order.indexOf("Resource boundary")).toBeLessThan(
    order.indexOf("gamma"),
  );
  expect(order.indexOf("gamma")).toBeLessThan(order.indexOf("change.txt"));
  expect(order.indexOf("change.txt")).toBeLessThan(order.indexOf("delta"));
});
