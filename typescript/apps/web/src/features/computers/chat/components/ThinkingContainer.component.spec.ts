import { expect, test } from "@playwright/test";
import superjson from "superjson";
import { containerInput, containerUser } from "./chatContainerFixture";
import type {
  ConversationHeader,
  ConversationRecord,
} from "@chapchoo/web-core/conversation-types";

test("current execution controls thinking across idle, attention, read failure and recovery", async ({
  page,
}) => {
  let foreground: ConversationHeader["foreground"] = "idle";
  let failed = false;
  let activityReads = 0;
  let requests: ConversationRecord[] = [];
  const unexpected: string[] = [];
  await page.route("**/api/trpc/**", async (route) => {
    const method = new URL(route.request().url()).pathname.split("/").at(-1);
    let value: unknown;
    switch (method) {
      case "auth.identity":
        value = containerUser;
        break;
      case "computers.get":
        if (containerInput.initial.type !== "detail") {
          throw new Error("Missing Agent fixture");
        }
        value = containerInput.initial.agent;
        break;
      case "conversations.activity":
        activityReads += 1;
        if (failed) {
          await route.fulfill({
            status: 500,
            json: {
              error: {
                json: {
                  message: "Activity unavailable",
                  code: -32603,
                  data: {
                    code: "INTERNAL_SERVER_ERROR",
                    path: method,
                    reason: null,
                    httpStatus: 500,
                  },
                },
              },
            },
          });
          return;
        }
        value = {
          ...containerInput.initialConversation,
          header: { ...containerInput.initialConversation.header, foreground },
          records: requests,
        };
        break;
      case "conversations.messages":
        value = containerInput.initialMessages;
        break;
      case "conversations.model":
        value = {};
        break;
      default:
        unexpected.push(method ?? route.request().url());
        await route.abort();
        return;
    }
    await route.fulfill({
      json: { result: { data: superjson.serialize(value) } },
    });
  });
  await page.goto(
    "/iframe.html?id=chat-container-transport--enqueue-failure&viewMode=story",
  );
  const thinking = page.getByRole("status").filter({ hasText: "Thinking" });
  await expect.poll(() => activityReads).toBeGreaterThan(0);
  await expect(thinking).toHaveCount(0);
  foreground = "working";
  await expect(thinking).toBeVisible();
  await expect(page.locator("[data-message-id]")).toHaveCount(0);
  for (const next of ["requiresAction", "idle"] as const) {
    foreground = next;
    await expect(thinking).toHaveCount(0);
    foreground = "working";
    await expect(thinking).toBeVisible();
  }
  requests = [
    {
      key: "request:approval",
      nativeId: "approval",
      kind: "request",
      anchor: { ordinal: 1n, index: 0n },
      updatedOrdinal: 1n,
      content: [],
      locations: [],
      responder: { token: "fixture-only" },
      question: {
        type: "permission",
        title: "Approve this work",
        content: [],
        locations: [],
        options: [{ id: "allow", title: "Allow", scope: "once" }],
      },
    },
  ];
  await expect(
    page.getByRole("region", { name: "Approve this work" }),
  ).toBeVisible();
  await expect(thinking).toHaveCount(0);
  requests = [];
  await expect(thinking).toBeVisible();
  failed = true;
  await expect(thinking).toHaveCount(0);
  failed = false;
  await expect(thinking).toBeVisible();
  expect(unexpected).toEqual([]);
});
