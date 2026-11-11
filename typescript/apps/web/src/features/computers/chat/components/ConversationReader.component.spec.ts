import { expect, test } from "@playwright/test";
import superjson from "superjson";
import type { ConversationRegion } from "@chapchoo/web-core/conversation-types";
import type { Route } from "@playwright/test";

function region(start: number, end: number): ConversationRegion {
  return {
    header: {
      agentId: "agent",
      conversationId: "conversation",
      state: "ready",
      foreground: "idle",
      source: {
        agentId: "agent",
        conversationId: "conversation",
        snapshotId: "snapshot",
      },
    },
    records: Array.from({ length: end - start + 1 }, (_, index) => ({
      key: `assistant:${start + index}`,
      nativeId: String(start + index),
      kind: "assistant",
      anchor: { ordinal: BigInt(start + index), index: 0n },
      updatedOrdinal: BigInt(start + index),
      content: [{ type: "text", text: `Message ${start + index}` }],
      locations: [],
    })),
    renderedAt: 1,
  };
}
async function reply(route: Route, value: ConversationRegion): Promise<void> {
  await route.fulfill({
    contentType: "application/json",
    body: JSON.stringify({ result: { data: superjson.serialize(value) } }),
  });
}

test("a page admitted during a tail poll executes and exhausted streams never revive", async ({
  page,
}) => {
  let receivePoll: (route: Route) => void = () => {};
  const poll = new Promise<Route>((resolve) => {
    receivePoll = resolve;
  });
  const requests: string[] = [];
  await page.route("**/api/trpc/conversations.region*", async (route) => {
    const body = route.request().postData() ?? "";
    requests.push(body);
    if (requests.length === 1) {
      receivePoll(route);
      return;
    }
    if (body.includes('"ordinal":"11"')) {
      await reply(route, {
        ...region(1, 10),
        inputOlder: { acceptedAt: 50n, requestId: "input50" },
      });
    } else {
      // No before means latest for that stream, not the start of another page chain.
      await reply(route, {
        ...region(17, 40),
        older: { ordinal: 17n, index: 0n },
      });
    }
  });
  await page.goto(
    "/iframe.html?id=chat-reader--automatic-pages&viewMode=story",
  );
  const log = page.getByRole("log", { name: "Full history", exact: true });
  await expect
    .poll(() => log.evaluate((element) => element.scrollTop))
    .toBeGreaterThan(0);
  await page.getByRole("button", { name: "Poll now" }).click();
  const held = await poll;
  await log.evaluate((element) => {
    element.scrollTop = 0;
  });
  await expect(page.getByTestId("reader-cursors")).toHaveText("11/100/loading");
  await reply(held, {
    ...region(17, 40),
    older: { ordinal: 17n, index: 0n },
    inputOlder: { acceptedAt: 100n, requestId: "input100" },
  });
  await expect(page.locator('[id="history-assistant%3A1"]')).toBeAttached();
  await log.evaluate((element) => {
    element.scrollTop = 0;
  });
  await expect(page.getByTestId("reader-cursors")).toHaveText("end/end/ready");
  await page.getByRole("button", { name: "Poll now" }).click();
  await expect.poll(() => requests.length).toBeGreaterThanOrEqual(4);
  await expect(page.getByTestId("reader-cursors")).toHaveText("end/end/ready");
  expect(
    requests.filter((body) => body.includes('"ordinal":"11"')),
  ).toHaveLength(1);
  expect(requests.some((body) => body.includes('"acceptedAt":"50"'))).toBe(
    true,
  );
});

test("failed older reads recover automatically on the same page cursor", async ({
  page,
}) => {
  let failed = false;
  let attempts = 0;
  await page.route("**/api/trpc/conversations.region*", async (route) => {
    if ((route.request().postData() ?? "").includes('"ordinal":"11"')) {
      attempts++;
      if (!failed) {
        failed = true;
        await route.abort();
        return;
      }
      await reply(route, region(1, 10));
    } else {
      await reply(route, {
        ...region(11, 40),
        older: { ordinal: 11n, index: 0n },
        inputOlder: { acceptedAt: 100n, requestId: "input100" },
      });
    }
  });
  await page.goto(
    "/iframe.html?id=chat-reader--automatic-pages&viewMode=story",
  );
  const log = page.getByRole("log", { name: "Full history", exact: true });
  await expect
    .poll(() => log.evaluate((element) => element.scrollTop))
    .toBeGreaterThan(0);
  await log.evaluate((element) => {
    element.scrollTop = 0;
  });
  await expect(page.getByRole("alert")).toContainText(
    "Conversation could not be updated",
  );
  await expect(page.getByTestId("reader-cursors")).toHaveText("end/end/ready");
  expect(attempts).toBe(2);
});

test("a persistently failing older page cannot starve new head events", async ({
  page,
}) => {
  let pageAttempts = 0;
  await page.route("**/api/trpc/conversations.region*", async (route) => {
    if ((route.request().postData() ?? "").includes('"ordinal":"11"')) {
      pageAttempts++;
      await route.abort();
    } else {
      await reply(route, {
        ...region(11, pageAttempts ? 41 : 40),
        older: { ordinal: 11n, index: 0n },
        inputOlder: { acceptedAt: 100n, requestId: "input100" },
      });
    }
  });
  await page.goto(
    "/iframe.html?id=chat-reader--automatic-pages&viewMode=story",
  );
  const log = page.getByRole("log", { name: "Full history", exact: true });
  await expect
    .poll(() => log.evaluate((element) => element.scrollTop))
    .toBeGreaterThan(0);
  await log.evaluate((element) => {
    element.scrollTop = 0;
  });
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(page.locator('[id="history-assistant%3A41"]')).toBeAttached();
  await expect(
    page.getByRole("button", { name: "New events below", exact: true }),
  ).toBeVisible();
  await expect.poll(() => pageAttempts).toBeGreaterThanOrEqual(2);
});
