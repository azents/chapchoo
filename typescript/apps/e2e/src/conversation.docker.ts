import { test, expect } from "@playwright/test";
import { Assembly } from "@chapchoo/testenv/assembly";
import { nativeConversation } from "@chapchoo/testenv/nativeConversation";

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
  page.on("response", (response) => {
    if (response.url().includes("conversations.messages")) messageReads++;
    if (response.url().includes("conversations.region")) historyReads++;
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
    stage = "inline-native-answer";
    await expect(
      page.getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      }),
    ).toBeVisible();
    await page.getByRole("radio", { name: "Stable", exact: true }).check();
    await page
      .getByRole("textbox", { name: "Confirmation note", exact: true })
      .fill("typed native answer");
    await page
      .getByRole("button", { name: "Send answers", exact: true })
      .click();
    const activeFixture = fixture;
    await expect.poll(() => activeFixture.pendingInputs()).toBe(0);
    await expect(
      page.getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      }),
    ).toHaveCount(0);
    await expect(log.locator("[data-message-id]")).toHaveCount(2);
    await expect(
      log.getByText(/First assistant output|Second assistant utterance/),
    ).toHaveCount(0);
    stage = "actual-full-records";
    await page
      .getByRole("button", { name: /^Conversation fixture.*Activity$/ })
      .click();
    await page
      .getByRole("button", { name: "Full history", exact: true })
      .click();
    const history = page.getByRole("log", {
      name: "Full history",
      exact: true,
    });
    await expect(
      history.locator('[data-record-origin="publication"]'),
    ).toHaveCount(1);
    const publication = history.locator('[data-record-origin="publication"]');
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
    expect(rendered).toHaveLength(2);
    expect(
      rendered.some((text) =>
        text?.includes("Intentional fixture publication."),
      ),
    ).toBe(true);
    expect(rendered.some((text) => text?.includes(inputText))).toBe(true);
    await expect(log.locator("[data-message-id]")).toHaveCount(2);
    await expect(
      log.getByText(/First assistant output|Second assistant utterance/),
    ).toHaveCount(0);
    const evidence = test.info();
    evidence.annotations.push({
      type: "conversation-read-evidence",
      description: `inputToPublicationMs=${inputToPublicationMs};messageReads=${messageReads};historyReads=${historyReads};ssrReceipts=${rendered.length}`,
    });
  } catch (error) {
    const locator =
      error instanceof Error
        ? error.message.split("\n").find((line) => line.startsWith("Locator:"))
        : undefined;
    failure = new Error(
      `Native conversation browser failed at ${stage}${locator ? ` (${locator})` : ""}`,
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
