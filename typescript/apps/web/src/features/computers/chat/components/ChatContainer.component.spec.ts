import { expect, test } from "@playwright/test";
import superjson from "superjson";
import {
  containerFailure,
  containerInput,
  containerUser,
  deferredContainerInput,
} from "./chatContainerFixture";
import type { Route } from "@playwright/test";

for (const width of [390, 1280]) {
  test(`model picker opens its own page and highlights the selection at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 844 });
    const current = { id: "fixture-a", label: "Fixture Model A" };
    const alternative = { id: "fixture-b", label: "Fixture Model B" };
    const saveRequested = Promise.withResolvers<void>();
    const approveSave = Promise.withResolvers<void>();
    let saved = current;
    let saves = 0;
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
            throw new Error("Missing fixture Agent");
          }
          value = containerInput.initial.agent;
          break;
        case "conversations.activity":
        case "conversations.region":
          value = containerInput.initialConversation;
          break;
        case "conversations.messages":
          value = containerInput.initialMessages;
          break;
        case "conversations.model":
          value = {
            target: {
              conversationId: "conversation",
              launchOperationId: "fixture-launch",
              native: { snapshotId: "snapshot", nativeSessionId: "native" },
            },
            current: saved,
            options: { choices: [current, alternative], permissionModes: [] },
          };
          break;
        case "conversations.saveModel": {
          const payload: unknown = route.request().postDataJSON();
          if (
            !payload ||
            typeof payload !== "object" ||
            !("json" in payload) ||
            !payload.json ||
            typeof payload.json !== "object" ||
            !("requestId" in payload.json) ||
            typeof payload.json.requestId !== "string" ||
            !("modelId" in payload.json) ||
            payload.json.modelId !== alternative.id
          ) {
            throw new Error(
              "Expected an explicit save for the alternative model",
            );
          }
          saves += 1;
          saveRequested.resolve();
          await approveSave.promise;
          saved = alternative;
          value = { requestId: payload.json.requestId, model: saved };
          break;
        }
        default:
          unexpected.push(method ?? "missing method");
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
    const open = page.getByRole("button", {
      name: width === 390 ? "Activity: Corgi" : "Activity",
      exact: true,
    });
    await open.click();
    const sheet = page.getByRole("dialog", { name: "Activity", exact: true });
    await expect(sheet.getByRole("link", { name: /Corgi/ })).toBeVisible();
    const model = sheet.getByRole("button", { name: /^Model/ });
    await expect(model).toContainText(current.label);
    await model.click();
    const picker = page.getByRole("dialog", {
      name: "Change model",
      exact: true,
    });
    const back = picker.getByRole("button", { name: "Back", exact: true });
    await expect(back).toBeFocused();
    await expect(page.getByRole("dialog")).toHaveCount(1);
    await expect(picker.getByRole("link", { name: /Corgi/ })).toHaveCount(0);
    const selected = picker.locator("label").filter({ hasText: current.label });
    const other = picker
      .locator("label")
      .filter({ hasText: alternative.label });
    await expect(selected.getByRole("radio")).toBeChecked();
    const highlight = await selected.evaluate(
      (row) => getComputedStyle(row).backgroundColor,
    );
    const plain = await other.evaluate(
      (row) => getComputedStyle(row).backgroundColor,
    );
    expect(highlight).not.toBe(plain);
    await expect(selected).toHaveCSS("font-weight", "600");
    const indicator = selected.locator("span[aria-hidden]");
    const [rowBox, square, glyph] = await Promise.all([
      selected.boundingBox(),
      indicator.boundingBox(),
      indicator.locator("svg").boundingBox(),
    ]);
    if (!rowBox || !square || !glyph) {
      throw new Error("Expected a visible square checkmark");
    }
    expect(square.y + square.height / 2).toBeCloseTo(
      rowBox.y + rowBox.height / 2,
      0,
    );
    expect(glyph.x + glyph.width / 2).toBeCloseTo(
      square.x + square.width / 2,
      0,
    );
    expect(glyph.y + glyph.height / 2).toBeCloseTo(
      square.y + square.height / 2,
      0,
    );
    await other.click();
    await expect(other.getByRole("radio")).toBeChecked();
    await expect(other).toHaveCSS("background-color", highlight);
    await expect(selected).toHaveCSS("background-color", plain);
    await back.click();
    await expect(model).toBeFocused();
    await expect(model).toContainText(current.label);
    await model.click();
    await expect(selected.getByRole("radio")).toBeChecked();
    await other.click();
    await picker.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(model).toContainText(current.label);
    expect(saves).toBe(0);
    await model.click();
    await other.click();
    await picker.getByRole("button", { name: "Save", exact: true }).click();
    await saveRequested.promise;
    try {
      await expect(back).toBeDisabled();
      await page.keyboard.press("Escape");
      await expect(picker).toBeVisible();
    } finally {
      approveSave.resolve();
    }
    await expect(model).toContainText(alternative.label);
    await expect(model).toBeFocused();
    expect(saves).toBe(1);
    expect(unexpected).toEqual([]);
  });
}

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
        case "conversations.model":
          value = {};
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
        case "conversations.model":
          value = {};
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
        case "conversations.model":
          value = {};
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
      case "conversations.model":
        return reply(route, {});
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
