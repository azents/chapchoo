import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";
import { Assembly } from "@chapchoo/testenv/assembly";
import { nativeConversation } from "@chapchoo/testenv/nativeConversation";

async function savePrompt(page: Page, content: string): Promise<void> {
  await page
    .getByRole("button", { name: "Edit system prompt", exact: true })
    .click();
  const editor = page.getByRole("dialog", {
    name: "System prompt",
    exact: true,
  });
  await editor
    .getByRole("textbox", { name: "Instructions", exact: true })
    .fill(content);
  await editor.getByRole("button", { name: "Save", exact: true }).click();
  await expect(editor).toHaveCount(0);
}

// Explicit disposable Docker prerequisites are required; no real model is used.
test("owner receives intentional server messages and inspects native typed-answer records", async ({
  browser,
}) => {
  test.setTimeout(660_000);
  const assembly = await Assembly.createDockerBrowser();
  let fixture: Awaited<ReturnType<typeof nativeConversation>> | null = null;
  const context = await browser.newContext({
    locale: "en-US",
    timezoneId: "UTC",
  });
  let stage = "prerequisite";
  let failure: Error | null = null;
  const page = await context.newPage();
  let messageReads = 0;
  let historyReads = 0;
  let sendRequests = 0;
  let sendStatus = 0;
  page.on("request", (request) => {
    if (request.url().includes("conversations.send")) sendRequests++;
    if (request.url().includes("conversations.region")) historyReads++;
  });
  page.on("response", (response) => {
    if (response.url().includes("conversations.messages")) messageReads++;
    if (response.url().includes("conversations.send"))
      sendStatus = response.status();
  });
  try {
    fixture = await nativeConversation(assembly);
    stage = "prestart-before-first-send";
    await expect
      .poll(
        async () => {
          const current = await fixture?.current();
          const target = current?.startup?.sessionReady;
          return Boolean(
            target &&
            current?.conversation?.snapshotId === target.snapshotId &&
            current.conversation.nativeSessionId === target.nativeSessionId,
          );
        },
        { timeout: 180_000 },
      )
      .toBe(true);
    const prestarted = await fixture.current();
    const prestartTarget = prestarted.startup?.sessionReady;
    expect(prestartTarget).toBeDefined();
    stage = "login";
    await page.goto(`${assembly.generalUrl}/login`);
    await page
      .getByRole("textbox", { name: /^(Email address|Email)$/ })
      .fill(assembly.administrator.email);
    await page
      .getByLabel(/^Password\s*\*?$/)
      .fill(assembly.administrator.password);
    await page.getByRole("button", { name: /^(Log in|Sign in)$/ }).click();
    await expect(
      page.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    stage = "open-chat";
    await page.goto(`${assembly.generalUrl}${fixture.chatPath}`);
    const message = page.getByRole("textbox", { name: "Message", exact: true });
    const log = page.getByRole("log", { name: "Message", exact: true });
    await expect(message).toBeEnabled({ timeout: 90_000 });
    expect(historyReads).toBe(0);
    const releaseWork = await fixture.holdWork();
    stage = "intentional-publication";
    const inputText = "Exercise the deterministic native conversation";
    await message.fill(inputText);
    const inputStart = performance.now();
    await page.getByRole("button", { name: "Send", exact: true }).click();
    await expect(
      log.getByText("Intentional fixture publication.", { exact: true }),
    ).toBeVisible({ timeout: 180_000 });
    await expect(log.locator("[data-message-id]")).toHaveCount(2);
    await expect(log.getByText(inputText, { exact: true })).toHaveCount(1);
    await expect(log.locator("[data-message-id]").first()).toContainText(
      inputText,
    );
    await expect(log.locator("[data-message-id]").last()).toContainText(
      "Intentional fixture publication.",
    );
    await expect(
      log.getByText("First assistant output", { exact: true }),
    ).toHaveCount(0);
    const inputToPublicationMs = Math.round(performance.now() - inputStart);
    const afterInput = await fixture.current();
    expect(afterInput.conversation?.conversationId).toBe(
      prestarted.conversation?.conversationId,
    );
    expect(afterInput.observedNative?.target).toEqual(prestartTarget);
    expect(historyReads).toBe(0);
    stage = "running-feedback";
    await expect(
      log.getByRole("status").filter({ hasText: "Thinking" }),
    ).toBeVisible();
    await releaseWork();
    stage = "inline-native-answer";
    await expect(
      page.getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      }),
    ).toBeVisible();
    await expect(
      log.getByRole("status").filter({ hasText: "Thinking" }),
    ).toHaveCount(0);
    stage = "save-and-queue-during-existing-work";
    const settingsPage = await context.newPage();
    await settingsPage.goto(
      `${assembly.generalUrl}${fixture.chatPath.replace(/\/chat$/, "")}`,
    );
    await savePrompt(settingsPage, "PROJECT_BETA");
    const activeFixture = fixture;
    await expect
      .poll(async () =>
        Boolean((await activeFixture.current()).startup?.possibleForwarding),
      )
      .toBe(true);
    await expect(message).toBeEnabled();
    const refreshInput = "Continue after refreshing project instructions";
    await message.fill(refreshInput);
    await page.getByRole("button", { name: "Send", exact: true }).click();
    await expect.poll(() => activeFixture.pendingInputs()).toBe(1);
    await expect(
      page.getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      }),
    ).toBeVisible();
    stage = "finish-existing-work-and-refresh";
    await page.getByRole("radio", { name: "Stable", exact: true }).check();
    await page
      .getByRole("textbox", { name: "Confirmation note", exact: true })
      .fill("typed native answer");
    stage = "complete-original-question";
    await page
      .getByRole("button", { name: "Send answers", exact: true })
      .click();
    stage = "await-refreshed-input-pickup";
    await expect
      .poll(
        async () => {
          const current = await activeFixture.current();
          if (current.startup?.failed !== void 0) {
            throw new Error("Native fixture configuration preparation failed");
          }
          return activeFixture.pendingInputs();
        },
        { timeout: 180_000 },
      )
      .toBe(0);
    stage = "await-refreshed-publication";
    await expect(
      log.getByText("Refreshed project fixture publication.", { exact: true }),
    ).toBeVisible({ timeout: 180_000 });
    await expect(log.getByText(refreshInput, { exact: true })).toHaveCount(1);
    const refreshed = await fixture.current();
    expect(refreshed.conversation?.nativeSessionId).toBe(
      prestartTarget?.nativeSessionId,
    );
    expect(refreshed.conversation?.snapshotId).toBe(prestartTarget?.snapshotId);
    await expect(
      page.getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      }),
    ).toHaveCount(0);
    await expect(log.locator("[data-message-id]")).toHaveCount(4);
    await expect(
      log.getByText(/First assistant output|Second assistant utterance/),
    ).toHaveCount(0);
    stage = "actual-full-records";
    await page
      .getByRole("button", { name: /^Conversation fixture.*Activity$/ })
      .click();
    stage = "latest-system-prompt-page";
    await page
      .getByRole("link", { name: "System prompt", exact: true })
      .click();
    await expect(page).toHaveURL(
      new RegExp(`${activeFixture.chatPath}/system-prompt$`),
    );
    await expect(
      page.getByRole("heading", { name: "System prompt", exact: true }),
    ).toBeVisible();
    const ownerSource = page.getByRole("region", {
      name: "Owner system prompt",
      exact: true,
    });
    await expect(ownerSource.locator("pre")).toHaveText("PROJECT_BETA");
    await savePrompt(settingsPage, "PROJECT_GAMMA");
    await expect
      .poll(
        async () => {
          const evidence = (await activeFixture.systemPrompt()).prompt;
          return (
            evidence !== undefined &&
            evidence.appliedAt !== void 0 &&
            evidence.sources.some((source) => source.text === "PROJECT_GAMMA")
          );
        },
        { timeout: 90_000 },
      )
      .toBe(true);
    await page.reload();
    await expect(ownerSource.locator("pre")).toHaveText("PROJECT_GAMMA", {
      timeout: 90_000,
    });
    await page
      .getByRole("link", { name: "Back to conversation", exact: true })
      .click();
    await expect(page).toHaveURL(new RegExp(`${activeFixture.chatPath}$`));
    await page
      .getByRole("button", { name: /^Conversation fixture.*Activity$/ })
      .click();
    stage = "actual-full-records";
    await page
      .getByRole("button", { name: "Full history", exact: true })
      .click();
    const history = page.getByRole("log", {
      name: "Full history",
      exact: true,
    });
    await expect(
      history.locator('[data-record-origin="publication"]'),
    ).toHaveCount(2);
    const publication = history
      .locator('[data-record-origin="publication"]')
      .first();
    await publication.getByRole("button", { expanded: false }).first().click();
    await expect(
      publication
        .locator("p")
        .filter({ hasText: /^Intentional fixture publication\.$/ }),
    ).toBeVisible();
    await expect(
      publication.getByText("Message accepted.", { exact: true }),
    ).toBeVisible();
    await publication
      .getByRole("button", { name: "Message input and result", exact: true })
      .click();
    await publication
      .locator("summary")
      .filter({ hasText: "Tool input" })
      .click();
    await expect(
      publication
        .locator("pre")
        .filter({ hasText: /"tool"\s*:\s*"send_message"/ }),
    ).toBeVisible();
    const outputs = history.locator('[data-record-origin="assistant"]');
    await expect(outputs).toHaveCount(2);
    await outputs
      .first()
      .getByRole("button", { expanded: false })
      .first()
      .click();
    await expect(
      outputs
        .first()
        .locator("p")
        .filter({
          hasText:
            /^First assistant output — updated after the typed answer\.$/,
        }),
    ).toBeVisible();
    await outputs
      .last()
      .getByRole("button", { expanded: false })
      .first()
      .click();
    await expect(
      outputs
        .last()
        .locator("p")
        .filter({
          hasText:
            /^Second assistant utterance: the native fixture received the exact typed answers\.$/,
        }),
    ).toBeVisible();
    const request = history.locator('[data-record-origin="request"]');
    await request.getByRole("button", { expanded: false }).first().click();
    await expect(
      history.getByText("typed native answer", { exact: true }),
    ).toBeVisible();
    await expect(
      history.getByRole("radio", { name: /Stable.*Selected/i }),
    ).toBeChecked();
    await page
      .getByRole("dialog", { name: "Full history", exact: true })
      .getByRole("button", { name: "Close", exact: true })
      .click();
    stage = "saved-versus-applied";
    await page
      .getByRole("button", { name: "Codex setup", exact: true })
      .click();
    const setup = page.getByRole("region", {
      name: "Codex setup",
      exact: true,
    });
    await expect(
      setup.getByRole("radio", {
        name: "Existing Codex installation",
        exact: true,
      }),
    ).toBeChecked();
    await expect(
      setup.getByText("/opt/chapchoo/codex-fixture", { exact: true }),
    ).toHaveCount(2);
    await setup
      .getByRole("radio", { name: "Managed preparation", exact: true })
      .check();
    await setup
      .getByRole("button", { name: "Save selection", exact: true })
      .click();
    await expect(
      setup.getByText("Selection saved", { exact: true }),
    ).toBeVisible();
    await expect(
      setup.getByText("/opt/chapchoo/codex-fixture", { exact: true }),
    ).toHaveCount(1);
    await expect(setup.getByRole("alert")).toContainText(
      "not on the next message",
    );
    await page
      .getByRole("dialog", { name: "Activity", exact: true })
      .getByRole("button", { name: "Close", exact: true })
      .click();
    stage = "reconnect";
    const reread = page.waitForResponse((response) =>
      response.url().includes("conversations.messages"),
    );
    await page.evaluate(() => window.dispatchEvent(new Event("online")));
    await reread;
    stage = "reentry-and-ssr";
    await page.goto(`${assembly.generalUrl}/teams/my/agents`);
    const response = await page.goto(
      `${assembly.generalUrl}${fixture.chatPath}`,
    );
    expect(response).not.toBeNull();
    const html = await response?.text();
    const rendered = await page.evaluate((source) => {
      const document = new DOMParser().parseFromString(
        source ?? "",
        "text/html",
      );
      return [...document.querySelectorAll("article[data-message-id]")].map(
        (element) => element.textContent,
      );
    }, html);
    expect(rendered).toHaveLength(4);
    expect(
      rendered.some((text) =>
        text?.includes("Intentional fixture publication."),
      ),
    ).toBe(true);
    expect(rendered.some((text) => text?.includes(inputText))).toBe(true);
    await expect(log.locator("[data-message-id]")).toHaveCount(4);
    await expect(
      log.getByText(/First assistant output|Second assistant utterance/),
    ).toHaveCount(0);
    const evidence = test.info();
    evidence.annotations.push({
      type: "conversation-read-evidence",
      description: `inputToPublicationMs=${inputToPublicationMs};messageReads=${messageReads};historyReads=${historyReads};ssrReceipts=${rendered.length}`,
    });
  } catch (error) {
    // Only selected numeric facts cross the private fixture boundary.
    let diagnostic = "unavailable";
    try {
      diagnostic = (await fixture?.diagnostics()) ?? "unavailable";
    } catch {
      // Diagnostic failure never replaces the original product failure.
    }
    const locator =
      error instanceof Error
        ? error.message.split("\n").find((line) => line.startsWith("Locator:"))
        : undefined;
    failure = new Error(
      `Native conversation browser failed at ${stage};sendRequests=${sendRequests};sendStatus=${sendStatus};messageReads=${messageReads};historyReads=${historyReads};${diagnostic}${locator ? ` (${locator})` : ""}`,
      { cause: error },
    );
  } finally {
    const cleanup: unknown[] = [];
    for (const close of [
      () => context.close(),
      () => fixture?.close(),
      () => assembly.close(),
    ]) {
      try {
        await close();
      } catch (error) {
        cleanup.push(error);
      }
    }
    if (cleanup.length)
      throw new AggregateError(
        [...(failure ? [failure] : []), ...cleanup],
        "Conversation acceptance cleanup was unconfirmed",
      );
  }
  if (failure) throw failure;
});

// Restored work scope deliberately cannot prove the empty inventory needed for
// project refresh. Verify deferred Send without changing that separate contract.
test("a stopped Cloud Computer restores its source and delivers an ordinary message once", async ({
  browser,
}) => {
  test.setTimeout(660_000);
  const assembly = await Assembly.createDockerBrowser();
  let fixture: Awaited<ReturnType<typeof nativeConversation>> | null = null;
  const context = await browser.newContext({
    locale: "en-US",
    timezoneId: "UTC",
  });
  const page = await context.newPage();
  let stage = "prerequisite";
  let failure: Error | null = null;
  let sends = 0;
  page.on("request", (request) => {
    if (request.url().includes("conversations.send")) sends++;
  });
  try {
    fixture = await nativeConversation(assembly);
    stage = "initial-source";
    await expect
      .poll(
        async () => {
          const current = await fixture?.current();
          return Boolean(
            current?.startup?.sessionReady &&
            current.conversation?.snapshotId ===
              current.startup.sessionReady.snapshotId &&
            current.conversation.nativeSessionId ===
              current.startup.sessionReady.nativeSessionId,
          );
        },
        { timeout: 180_000 },
      )
      .toBe(true);
    const before = await fixture.current();
    const target = before.startup?.sessionReady;
    expect(target).toBeDefined();
    stage = "login";
    await page.goto(`${assembly.generalUrl}/login`);
    await page
      .getByRole("textbox", { name: /^(Email address|Email)$/ })
      .fill(assembly.administrator.email);
    await page
      .getByLabel(/^Password\s*\*?$/)
      .fill(assembly.administrator.password);
    await page.getByRole("button", { name: /^(Log in|Sign in)$/ }).click();
    await expect(
      page.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    await page.goto(`${assembly.generalUrl}${fixture.chatPath}`);
    const message = page.getByRole("textbox", { name: "Message", exact: true });
    const log = page.getByRole("log", { name: "Message", exact: true });
    stage = "stop-before-send";
    await page
      .getByRole("button", { name: /Activity/ })
      .filter({ has: page.locator("strong") })
      .click();
    const activity = page.getByRole("dialog", {
      name: "Activity",
      exact: true,
    });
    await activity.getByRole("button", { name: "Stop", exact: true }).click();
    await page
      .getByRole("dialog", { name: "Stop", exact: true })
      .getByRole("button", { name: "Confirm", exact: true })
      .click();
    await expect(
      activity.getByText("Computer · Stopped", { exact: true }),
    ).toBeVisible({ timeout: 180_000 });
    await activity.getByRole("button", { name: "Close", exact: true }).click();
    await expect(activity).toBeHidden();
    await expect(message).toBeEnabled();
    await expect(message).toHaveAttribute("aria-placeholder", /Send a message/);
    stage = "ordinary-send-and-automatic-start";
    const text = "Exercise the deterministic native conversation";
    await message.fill(text);
    await page.getByRole("button", { name: "Send", exact: true }).click();
    await expect(
      log.getByText("Intentional fixture publication.", { exact: true }),
    ).toBeVisible({ timeout: 180_000 });
    await expect(log.getByText(text, { exact: true })).toHaveCount(1);
    await expect(log.locator("[data-message-id]")).toHaveCount(2);
    expect(sends).toBe(1);
    const after = await fixture.current();
    expect(after.conversation?.conversationId).toBe(
      before.conversation?.conversationId,
    );
    expect(after.observedNative?.target).toEqual(target);
    expect(await fixture.pendingInputs()).toBe(0);
    // Finish the deterministic fixture's question so cleanup owns no waiting
    // responder. This does not attempt unsupported restored-context refresh.
    stage = "finish-owned-fixture-work";
    await page
      .getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      })
      .waitFor();
    await page.getByRole("radio", { name: "Stable", exact: true }).check();
    await page
      .getByRole("textbox", { name: "Confirmation note", exact: true })
      .fill("typed native answer");
    await page
      .getByRole("button", { name: "Send answers", exact: true })
      .click();
    await expect(
      page.getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      }),
    ).toHaveCount(0);
    await expect(log.locator("[data-message-id]")).toHaveCount(2);
  } catch (error) {
    let diagnostic = "unavailable";
    try {
      diagnostic = (await fixture?.diagnostics()) ?? "unavailable";
    } catch {
      /* Preserve the original failure. */
    }
    failure = new Error(
      `Stopped-send acceptance failed at ${stage};sends=${sends};${diagnostic}`,
      { cause: error },
    );
  } finally {
    const cleanup: unknown[] = [];
    for (const close of [
      () => context.close(),
      () => fixture?.close(),
      () => assembly.close(),
    ]) {
      try {
        await close();
      } catch (error) {
        cleanup.push(error);
      }
    }
    if (cleanup.length)
      throw new AggregateError(
        [...(failure ? [failure] : []), ...cleanup],
        "Stopped-send acceptance cleanup was unconfirmed",
      );
  }
  if (failure) throw failure;
});
