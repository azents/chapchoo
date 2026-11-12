import { expect, test } from "@playwright/test";
import superjson from "superjson";
import type { ConversationMessageRegion } from "@chapchoo/web-core/conversation-message-types";
import type { Route } from "@playwright/test";

function region(
  start: number,
  end: number,
  through = end,
): ConversationMessageRegion {
  return {
    agentId: "agent",
    conversationId: "conversation",
    messages: Array.from(
      { length: Math.max(0, end - start + 1) },
      (_, index) => ({
        id: `message-${start + index}`,
        role: "agent",
        order: BigInt(start + index),
        receivedAt: BigInt(start + index),
        text: `Receipt ${start + index}`,
      }),
    ),
    window: { type: "after", order: BigInt(start - 1) },
    through: BigInt(through),
    renderedAt: 2,
  };
}
async function reply(
  route: Route,
  value: ConversationMessageRegion,
): Promise<void> {
  await route.fulfill({
    contentType: "application/json",
    body: JSON.stringify({ result: { data: superjson.serialize(value) } }),
  });
}
const story =
  "/iframe.html?id=chat-message-reader--continuation&viewMode=story";

for (const edited of [false, true]) {
  test(`lost send response and consumed input acknowledge one receipt without clearing newer edits: ${edited}`, async ({
    page,
  }) => {
    let received: (route: Route) => void = () => {};
    const held = new Promise<Route>((resolve) => {
      received = resolve;
    });
    await page.route("**/api/trpc/conversations.messages*", (route) => {
      received(route);
    });
    await page.goto(
      "/iframe.html?id=chat-message-reader--lost-response&viewMode=story",
    );
    const draft = page.getByRole("textbox", { name: "Draft", exact: true });
    if (edited) {
      await draft.fill("A newer unsent edit");
    }
    await page.getByRole("button", { name: "Poll now", exact: true }).click();
    const pending = await held;
    const accepted = region(50, 50);
    await reply(pending, {
      ...accepted,
      messages: [
        {
          id: "receipt",
          role: "user",
          order: 50n,
          receivedAt: 50n,
          text: "Accepted SSR receipt",
          sourceInputId: "consumed-input",
          sourceInputRequestId: "nonce",
        },
      ],
    });
    await expect(
      page.getByText("Accepted SSR receipt", { exact: true }),
    ).toHaveCount(1);
    await expect(page.locator("[data-message-id]")).toHaveCount(1);
    await expect(draft).toHaveValue(edited ? "A newer unsent edit" : "");
    await expect(
      page.getByText("Response not confirmed", { exact: true }),
    ).toHaveCount(0);
  });
}

test("exact hydration and finite receipt catch-up never read native history", async ({
  page,
}) => {
  await page.clock.install();
  await page.clock.pauseAt(new Date());
  const calls: string[] = [];
  const native: string[] = [];
  await page.route("**/api/trpc/conversations.region*", (route) => {
    native.push(route.request().url());
    return route.abort();
  });
  await page.route("**/api/trpc/conversations.messages*", async (route) => {
    const body = route.request().postData() ?? "";
    calls.push(body);
    if (body.includes('"order":"75"')) {
      await reply(route, region(76, 100));
    } else {
      await reply(route, {
        ...region(51, 75, 100),
        next: { order: 75n, through: 100n },
      });
    }
  });
  await page.goto(story);
  await expect(
    page.getByText("Accepted SSR receipt", { exact: true }),
  ).toHaveCount(1);
  await expect(
    page.getByText("Ordinary native output stays in full records"),
  ).toHaveCount(0);
  expect(calls).toHaveLength(0);
  await page.clock.runFor(1000);
  await expect(page.getByTestId("receipt-state")).toHaveText(
    "conversation/100/75/50/ready/ready",
  );
  await page.clock.runFor(1);
  await expect(page.getByTestId("receipt-state")).toHaveText(
    "conversation/100/end/50/ready/ready",
  );
  expect(calls).toHaveLength(2);
  expect(calls[1]).toContain('"through":"100"');
  await expect(page.locator("[data-message-id]")).toHaveCount(51);
  expect(native).toHaveLength(0);
});

test("latest intent supersedes an in-flight older response without merging a gap", async ({
  page,
}) => {
  await page.clock.install();
  await page.clock.pauseAt(new Date());
  let received: (route: Route) => void = () => {};
  const held = new Promise<Route>((resolve) => {
    received = resolve;
  });
  await page.route("**/api/trpc/conversations.messages*", async (route) => {
    const body = route.request().postData() ?? "";
    if (body.includes('"type":"before"')) {
      received(route);
      return;
    }
    await reply(route, {
      ...region(101, 125),
      window: { type: "latest" },
      older: 101n,
    });
  });
  await page.goto(story);
  await page.getByRole("button", { name: "Older", exact: true }).click();
  const pending = await held;
  await page.getByRole("button", { name: "Latest", exact: true }).click();
  await reply(pending, {
    ...region(25, 49),
    window: { type: "before", order: 50n },
    older: 25n,
  });
  await page.clock.resume();
  await expect(page.getByTestId("receipt-state")).toHaveText(
    "conversation/125/end/101/ready/ready",
  );
  await expect(page.getByText("Receipt 25", { exact: true })).toHaveCount(0);
  await expect(
    page.getByText("Accepted SSR receipt", { exact: true }),
  ).toHaveCount(0);
  await expect(page.locator("[data-message-id]")).toHaveCount(25);
});

test("withdrawal discards an in-flight response and a new owner continues after restoration", async ({
  page,
}) => {
  await page.clock.install();
  await page.clock.pauseAt(new Date());
  let received: (route: Route) => void = () => {};
  const held = new Promise<Route>((resolve) => {
    received = resolve;
  });
  let calls = 0;
  await page.route("**/api/trpc/conversations.messages*", async (route) => {
    if (++calls === 1) {
      received(route);
      return;
    }
    await reply(route, region(51, 51));
  });
  await page.goto(story);
  await page.getByRole("button", { name: "Poll now", exact: true }).click();
  const pending = await held;
  await page
    .getByRole("button", { name: "Withdraw access", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Restore access", exact: true })
    .click();
  await page.getByRole("button", { name: "Poll now", exact: true }).click();
  await reply(pending, region(52, 52));
  await expect(page.getByText("Receipt 51", { exact: true })).toBeVisible();
  await expect(page.getByText("Receipt 52", { exact: true })).toHaveCount(0);
});

test("older failure yields to live head and then retries the exact older cursor", async ({
  page,
}) => {
  let older = 0;
  const requests: string[] = [];
  await page.route("**/api/trpc/conversations.messages*", async (route) => {
    const body = route.request().postData() ?? "";
    requests.push(body);
    if (body.includes('"type":"before"')) {
      if (++older === 1) {
        await route.abort();
        return;
      }
      await reply(route, {
        ...region(25, 49, 51),
        window: { type: "before", order: 50n },
      });
    } else {
      await reply(route, region(51, 51));
    }
  });
  await page.goto(story);
  await page.getByRole("button", { name: "Older", exact: true }).click();
  await expect(page.getByTestId("receipt-state")).toContainText("/error");
  await expect(page.getByText("Receipt 25", { exact: true })).toBeVisible();
  await expect(page.getByTestId("receipt-state")).toHaveText(
    "conversation/51/end/end/ready/ready",
  );
  expect(older).toBe(2);
  expect(requests[1]).toContain('"type":"after"');
});
