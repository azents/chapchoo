import { expect, test } from "@playwright/test";
import superjson from "superjson";
import type { Page } from "@playwright/test";
async function mockFlow(page: Page, uncertain: boolean): Promise<string[]> {
  const calls: string[] = [];
  await page.route("**/api/trpc/authentications.*", async (route) => {
    const method =
      new URL(route.request().url()).pathname.split(".").at(-1) ?? "";
    calls.push(method);
    let result: unknown;
    if (method === "list") {
      result = { items: [], nextAfter: null };
    } else if (method === "startDevice") {
      result = {
        attemptId: "attempt",
        expiresAt: Date.now() + 300000,
        state: {
          type: "pending",
          userCode: "ABCD-EFGH",
          verificationUri: "https://auth.openai.com/codex/device",
          nextCheckAt: 1,
        },
        providerFailure: null,
      };
    } else if (method === "pollDevice") {
      if (uncertain) {
        await route.abort("failed");
        return;
      }
      result = {
        attemptId: "attempt",
        expiresAt: Date.now() + 300000,
        state: { type: "registered", authenticationId: "connected" },
        providerFailure: null,
      };
    } else if (method === "deviceStatus") {
      result = {
        attemptId: "attempt",
        expiresAt: Date.now() + 300000,
        state: { type: "registered", authenticationId: "connected" },
        providerFailure: null,
      };
    } else if (method === "get") {
      result = {
        id: "connected",
        scope: { type: "personal" },
        kind: "chatgpt",
        name: "Connected fixture",
        accountId: "synthetic",
        email: null,
        planType: null,
        expiresAt: null,
        currentRequestId: "registered",
        createdAt: 1,
        updatedAt: 1,
      };
    } else {
      throw new Error("Unexpected authentication browser fixture method");
    }
    await route.fulfill({
      json: { result: { data: superjson.serialize(result) } },
    });
  });
  return calls;
}
async function start(page: Page): Promise<void> {
  await page.goto(
    "/iframe.html?id=authentication-flow--in-place&viewMode=story",
  );
  await page
    .getByRole("button", { name: "Connect account", exact: true })
    .first()
    .click();
  await page
    .getByRole("button", { name: "ChatGPT account", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Connection name", exact: true })
    .fill("Device fixture");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Connect account", exact: true })
    .click();
  await expect(page.getByText("ABCD-EFGH", { exact: true })).toBeVisible();
}
test("successful device exchange returns directly to the owning selection without another registration", async ({
  page,
}) => {
  const calls = await mockFlow(page, false);
  await start(page);
  await page
    .getByRole("button", { name: "Check connection", exact: true })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "Selected: Connected fixture",
  );
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(calls.filter((value) => value === "startDevice")).toHaveLength(1);
  expect(calls.filter((value) => value === "pollDevice")).toHaveLength(1);
});
test("a lost explicit device advance switches to status recovery without replay", async ({
  page,
}) => {
  const calls = await mockFlow(page, true);
  await start(page);
  await page
    .getByRole("button", { name: "Check connection", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Check connection", exact: true }),
  ).toHaveCount(0);
  expect(calls.filter((value) => value === "pollDevice")).toHaveLength(1);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Reload", exact: true })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "Selected: Connected fixture",
  );
  expect(calls.filter((value) => value === "pollDevice")).toHaveLength(1);
  expect(calls.filter((value) => value === "deviceStatus")).toHaveLength(1);
});

test("lost cancellation with no receipt observes its known attempt without replay", async ({
  page,
}) => {
  const calls = await mockFlow(page, false);
  let cancels = 0;
  await page.route(
    "**/api/trpc/authentications.cancelDevice",
    async (route) => {
      cancels += 1;
      await route.abort("failed");
    },
  );
  await page.route(
    "**/api/trpc/authentications.requestResult",
    async (route) => {
      await route.fulfill({
        json: {
          result: { data: superjson.serialize({ type: "notObserved" }) },
        },
      });
    },
  );
  await start(page);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Cancel", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Check request result", exact: true })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "Selected: Connected fixture",
  );
  expect(cancels).toBe(1);
  expect(calls.filter((value) => value === "deviceStatus")).toHaveLength(1);
  expect(calls.filter((value) => value === "pollDevice")).toHaveLength(0);
});

test("accepted key followed by an unavailable read retains its original receipt rather than registering twice", async ({
  page,
}) => {
  await mockFlow(page, false);
  let registrations = 0;
  let reads = 0;
  let acceptedRequest: string | null = null;
  await page.route("**/api/trpc/authentications.registerKey", async (route) => {
    registrations += 1;
    const body: unknown = route.request().postDataJSON();
    if (
      !body ||
      typeof body !== "object" ||
      !("json" in body) ||
      !body.json ||
      typeof body.json !== "object" ||
      !("requestId" in body.json) ||
      typeof body.json.requestId !== "string"
    ) {
      throw new Error("A fixture Request ID is required");
    }
    acceptedRequest = body.json.requestId;
    await route.fulfill({
      json: {
        result: {
          data: superjson.serialize({
            type: "record",
            authenticationId: "connected",
          }),
        },
      },
    });
  });
  await page.route("**/api/trpc/authentications.get", async (route) => {
    reads += 1;
    if (reads === 1) {
      await route.fulfill({
        status: 503,
        json: {
          error: {
            json: {
              message: "Unavailable",
              code: -32603,
              data: {
                code: "INTERNAL_SERVER_ERROR",
                path: "authentications.get",
                reason: null,
                httpStatus: 503,
              },
            },
          },
        },
      });
    } else {
      await route.fallback();
    }
  });
  await page.route(
    "**/api/trpc/authentications.requestResult",
    async (route) => {
      const body: unknown = route.request().postDataJSON();
      if (
        !body ||
        typeof body !== "object" ||
        !("json" in body) ||
        !body.json ||
        typeof body.json !== "object" ||
        !("requestId" in body.json)
      ) {
        throw new Error("A fixture recovery Request ID is required");
      }
      expect(body.json.requestId).toBe(acceptedRequest);
      await route.fulfill({
        json: {
          result: {
            data: superjson.serialize({
              type: "accepted",
              operation: "registerKey",
              result: { type: "record", authenticationId: "connected" },
              acceptedAt: 1,
            }),
          },
        },
      });
    },
  );
  await page.goto(
    "/iframe.html?id=authentication-flow--in-place&viewMode=story",
  );
  await page
    .getByRole("button", { name: "Connect account", exact: true })
    .first()
    .click();
  await page
    .getByRole("button", { name: "OpenAI API key", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Connection name", exact: true })
    .fill("Synthetic key");
  await page.getByLabel(/^API key\s*\*?$/).fill("disposable-component-key");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await page
    .getByRole("button", { name: "Check request result", exact: true })
    .click();
  await expect(page.getByRole("status")).toHaveText(
    "Selected: Connected fixture",
  );
  expect(registrations).toBe(1);
  expect(reads).toBe(2);
});

test("accepted record that is now unavailable returns to its task without claiming rejection or re-registering", async ({
  page,
}) => {
  const calls = await mockFlow(page, false);
  await page.route("**/api/trpc/authentications.get", async (route) => {
    await route.fulfill({
      status: 404,
      json: {
        error: {
          json: {
            message: "Not found",
            code: -32004,
            data: {
              code: "NOT_FOUND",
              path: "authentications.get",
              reason: "DOMAIN_REASON_NOT_FOUND",
              httpStatus: 404,
            },
          },
        },
      },
    });
  });
  await start(page);
  await page
    .getByRole("button", { name: "Check connection", exact: true })
    .click();
  await expect(
    page.getByText(
      "The connection was saved, but is no longer available to select. No replacement was selected.",
      { exact: true },
    ),
  ).toBeVisible();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Close", exact: true })
    .filter({ hasText: "Close" })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByRole("status")).toHaveText("Selected: ");
  expect(calls.filter((value) => value === "startDevice")).toHaveLength(1);
  expect(calls.filter((value) => value === "pollDevice")).toHaveLength(1);
});
