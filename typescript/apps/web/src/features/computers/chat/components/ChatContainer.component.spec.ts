import { expect, test } from "@playwright/test";
import superjson from "superjson";
import {
  containerFailure,
  containerInput,
  containerUser,
  deferredContainerInput,
} from "./chatContainerFixture";
import type { Route } from "@playwright/test";

for (const state of ["stopped", "starting", "restoring"] as const) {
  test(`ordinary message submission remains available while Computer is ${state}`, async ({
    page,
  }) => {
    const initial = deferredContainerInput(state);
    const sends: unknown[] = [];
    const unexpected: string[] = [];
    await page.route("**/api/trpc/**", async (route) => {
      const method = new URL(route.request().url()).pathname.split("/").at(-1);
      let value: unknown;
      switch (method) {
        case "auth.identity":
          value = containerUser;
          break;
        case "computers.get":
          if (initial.initial.type !== "detail") {
            throw new Error("Missing fixture Agent");
          }
          value = initial.initial.agent;
          break;
        case "conversations.activity":
        case "conversations.region":
          value = initial.initialConversation;
          break;
        case "conversations.messages":
          value = initial.initialMessages;
          break;
        case "conversations.send":
          sends.push(route.request().postDataJSON());
          value = void 0;
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
      `/iframe.html?id=chat-container-transport--${state}&viewMode=story`,
    );
    const input = page.getByRole("textbox", { name: "Message", exact: true });
    await expect(input).toBeEnabled();
    await expect(input).toHaveAttribute("aria-placeholder", /Corgi ·/);
    const text = `  ${state}: 준비되면 전달 🙂\\n `;
    await input.fill(text);
    await page.getByRole("button", { name: "Send", exact: true }).click();
    await expect(input).toHaveText("");
    await expect(input).toBeFocused();
    await expect(
      page.getByRole("button", { name: "Stop", exact: true }),
    ).toHaveCount(0);
    expect(sends).toHaveLength(1);
    expect(sends[0]).toMatchObject({ json: { text } });
    expect(unexpected).toEqual([]);
  });
}

for (const width of [390, 1280]) {
  test(`recent-record error offers adjacent read-only recovery at ${width}`, async ({
    page,
  }, testInfo) => {
    await page.setViewportSize({ width, height: 844 });
    let recovered = false;
    let reads = 0;
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
            throw new Error("Missing Agent");
          }
          value = containerInput.initial.agent;
          break;
        case "conversations.activity":
          value = containerInput.initialConversation;
          break;
        case "conversations.messages":
          value = containerInput.initialMessages;
          break;
        case "conversations.region":
          reads += 1;
          if (!recovered) {
            await route.fulfill({
              status: 500,
              json: {
                error: {
                  json: {
                    message: "Unavailable",
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
          value = containerInput.initialConversation;
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
    await page.getByRole("button", { name: /^Activity:/ }).click();
    const dialog = page.getByRole("dialog");
    const alert = dialog.getByRole("alert");
    await expect(alert).toHaveText(/Could not load conversation records/);
    const retry = alert.getByRole("button", {
      name: "Read again",
      exact: true,
    });
    await expect(retry).toBeEnabled();
    await retry.scrollIntoViewIfNeeded();
    await expect(alert).toBeVisible();
    await page.screenshot({
      path: testInfo.outputPath("record-error.png"),
      animations: "disabled",
    });
    recovered = true;
    await retry.focus();
    await retry.press("Enter");
    await expect(alert).toHaveCount(0);
    expect(reads).toBeGreaterThanOrEqual(2);
    expect(unexpected).toEqual([]);
  });
}

for (const outcome of ["failed", "accepted"] as const) {
  test(`actual Chat container preserves an in-flight draft until enqueue is ${outcome}`, async ({
    page,
  }) => {
    const sends: unknown[] = [];
    const admitted = Promise.withResolvers<void>();
    const release = Promise.withResolvers<void>();
    const unexpected: string[] = [];
    await page.route("**/api/trpc/**", async (route) => {
      const method = new URL(route.request().url()).pathname.split("/").at(-1);
      if (method === "conversations.send") {
        sends.push(route.request().postDataJSON());
        admitted.resolve();
        await release.promise;
        if (outcome === "failed") {
          await route.abort("failed");
        } else {
          await route.fulfill({
            contentType: "application/json",
            body: JSON.stringify({
              result: { data: superjson.serialize(void 0) },
            }),
          });
        }
        return;
      }
      let value: unknown;
      switch (method) {
        case "auth.identity":
          value = containerUser;
          break;
        case "computers.get":
          if (containerInput.initial.type !== "detail") {
            throw new Error("Fixture requires an Agent detail");
          }
          value = containerInput.initial.agent;
          break;
        case "conversations.region":
        case "conversations.activity":
          value = containerInput.initialConversation;
          break;
        case "conversations.messages":
          value = containerInput.initialMessages;
          break;
        default:
          unexpected.push(method ?? route.request().url());
          await route.abort();
          return;
      }
      await route.fulfill({
        contentType: "application/json",
        body: JSON.stringify({ result: { data: superjson.serialize(value) } }),
      });
    });
    await page.goto(
      "/iframe.html?id=chat-container-transport--enqueue-failure&viewMode=story",
    );
    const input = page.getByRole("textbox", { name: "Message", exact: true });
    await expect(input).toBeEnabled();
    const draft = "Keep this exact failed draft 🙂";
    await input.fill(draft);
    await input.evaluate((node) => {
      node.setAttribute("data-blurs", "0");
      node.addEventListener("blur", () =>
        node.setAttribute(
          "data-blurs",
          String(Number(node.getAttribute("data-blurs")) + 1),
        ),
      );
    });
    await page.getByRole("button", { name: "Send", exact: true }).click();
    await admitted.promise;
    await expect(input).toBeEnabled();
    await expect(input).toBeFocused();
    await expect(input).toHaveAttribute("aria-readonly", "true");
    await expect(input).toHaveAttribute("data-blurs", "0");
    await expect(
      page.getByRole("button", { name: "Send", exact: true }),
    ).toBeDisabled();
    await input.press("End");
    await input.pressSequentially(" must not alter the pending draft");
    await expect(input).toHaveText(draft);
    await input.press("Enter");
    release.resolve();
    if (outcome === "failed") {
      await expect(page.locator("#chat-input-error")).toBeVisible();
      await expect(input).toHaveText(draft);
    } else {
      await expect(page.locator("#chat-input-error")).toHaveCount(0);
      await expect(input).toHaveText("");
    }
    await expect(input).toBeFocused();
    await expect(input).toHaveAttribute("aria-readonly", "false");
    await expect(input).toHaveAttribute("data-blurs", "0");
    await expect(page.locator("[data-input-operation]")).toHaveCount(0);
    await expect(page.locator("[data-user-bubble]")).toHaveCount(0);
    expect(sends).toHaveLength(1);
    expect(unexpected).toEqual([]);
  });
}

test("Delete removes the queue item and invalidates only the earlier in-flight read", async ({
  page,
}) => {
  let deleted = false;
  let reads = 0;
  const captured = Promise.withResolvers<Route>();
  const released = Promise.withResolvers<void>();
  const unexpected: string[] = [];
  async function reply(route: Route, value: unknown): Promise<void> {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ result: { data: superjson.serialize(value) } }),
    });
  }
  await page.route("**/api/trpc/**", async (route) => {
    const method = new URL(route.request().url()).pathname.split("/").at(-1);
    switch (method) {
      case "auth.identity":
        return reply(route, containerUser);
      case "computers.get":
        if (containerInput.initial.type !== "detail") {
          throw new Error("Missing Agent");
        }
        return reply(route, containerInput.initial.agent);
      case "conversations.messages":
        return reply(route, containerInput.initialMessages);
      case "conversations.region":
        return reply(route, containerInput.initialConversation);
      case "conversations.deleteInput":
        deleted = true;
        return reply(route, void 0);
      case "conversations.activity":
        reads++;
        if (reads === 1) {
          captured.resolve(route);
          await released.promise;
          return reply(route, containerFailure.initialConversation);
        }
        return reply(
          route,
          deleted
            ? containerInput.initialConversation
            : containerFailure.initialConversation,
        );
      default:
        unexpected.push(method ?? route.request().url());
        return route.abort();
    }
  });
  await page.goto(
    "/iframe.html?id=chat-container-transport--removal-race&viewMode=story",
  );
  await expect(
    page.getByText("Retained failed queue item", { exact: true }),
  ).toBeVisible();
  await captured.promise;
  await page.getByRole("button", { name: "Delete", exact: true }).click();
  await expect(page.locator("[data-input-operation]")).toHaveCount(0);
  released.resolve();
  await expect.poll(() => reads).toBeGreaterThanOrEqual(2);
  await expect(page.locator("[data-input-operation]")).toHaveCount(0);
  await expect(
    page.getByRole("textbox", { name: "Message", exact: true }),
  ).toBeEnabled();
  expect(unexpected).toEqual([]);
});
