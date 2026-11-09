import { test, expect } from "@playwright/test";
import { Assembly } from "@chapchoo/testenv/assembly";
import { nativeConversation } from "@chapchoo/testenv/nativeConversation";

// The explicit disposable Docker inputs are required; missing prerequisites fail.
test("owner chats through the official adapter and retains the native typed answer", async ({
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
  let regionReads = 0;
  let lastReadCompleted = 0;
  page.on("response", (response) => {
    if (response.url().includes("conversations.region")) {
      regionReads++;
      void response
        .finished()
        .then(() => {
          lastReadCompleted = performance.now();
        })
        .catch(() => undefined);
    }
  });
  try {
    fixture = await nativeConversation(assembly);
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
    await expect(message).toBeEnabled({ timeout: 90_000 });
    const initialReads = regionReads;
    stage = "owner-supply";
    await page
      .getByRole("button", { name: /^Conversation fixture.*Activity$/ })
      .click();
    await page
      .getByRole("button", { name: "Codex setup", exact: true })
      .click();
    const setup = page.getByRole("region", {
      name: "Codex setup",
      exact: true,
    });
    await expect(
      setup
        .getByRole("region", {
          name: "Observed applied selection",
          exact: true,
        })
        .getByText("Not observed", { exact: true }),
    ).toBeVisible();
    await setup
      .getByRole("radio", { name: "Existing Codex installation", exact: true })
      .check();
    await setup
      .getByRole("textbox", { name: "Codex executable path", exact: true })
      .fill("/opt/chapchoo/codex-fixture");
    await setup
      .getByRole("button", { name: "Save selection", exact: true })
      .click();
    await expect(
      setup.getByText("Selection saved", { exact: true }),
    ).toBeVisible();
    await page
      .getByRole("dialog", { name: "Activity", exact: true })
      .getByRole("button", { name: "Close", exact: true })
      .click();
    await message.fill("Exercise the deterministic native conversation");
    const inputStart = performance.now();
    await page.getByRole("button", { name: "Send", exact: true }).click();
    stage = "first-assistant";
    await expect(
      page.getByText("First assistant output", { exact: true }),
    ).toBeVisible({ timeout: 180_000 });
    const inputToFirstMs = Math.round(performance.now() - inputStart);
    stage = "inline-request";
    await expect(
      page.getByRole("heading", {
        name: "Review the native fixture choices",
        exact: true,
      }),
    ).toBeVisible();
    await expect(
      page.getByRole("radio", { name: "Stable", exact: true }),
    ).not.toBeChecked();
    stage = "inline-choice";
    await page.getByRole("radio", { name: "Stable", exact: true }).check();
    stage = "inline-note";
    await page
      .getByRole("textbox", { name: "Confirmation note", exact: true })
      .fill("typed native answer");
    const answerStart = performance.now();
    await page
      .getByRole("button", { name: "Send answers", exact: true })
      .click();
    stage = "updated-assistant";
    await expect(
      page.getByText(
        "First assistant output — updated after the typed answer.",
        { exact: true },
      ),
    ).toBeVisible();
    const answerToUpdateMs = Math.round(performance.now() - answerStart);
    const responseToVisibleUpperMs = Math.max(
      0,
      Math.round(performance.now() - lastReadCompleted),
    );
    await expect(
      page.getByText(
        "Second assistant utterance: the native fixture received the exact typed answers.",
        { exact: true },
      ),
    ).toBeVisible();
    await expect(
      page.getByText("First assistant output", { exact: true }),
    ).toHaveCount(0);
    stage = "saved-versus-applied";
    await page
      .getByRole("button", { name: /^Conversation fixture.*Activity$/ })
      .click();
    const setupToggle = page.getByRole("button", {
      name: "Codex setup",
      exact: true,
    });
    if ((await setupToggle.getAttribute("aria-expanded")) !== "true") {
      await setupToggle.click();
    }
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
    const reconnectStart = performance.now();
    const reread = page.waitForResponse((response) =>
      response.url().includes("conversations.region"),
    );
    await page.evaluate(() => window.dispatchEvent(new Event("online")));
    await reread;
    const reconnectMs = Math.round(performance.now() - reconnectStart);
    stage = "reentry";
    await page.goto(`${assembly.generalUrl}/teams/my/agents`);
    const reentryStart = performance.now();
    await page.goto(`${assembly.generalUrl}${fixture.chatPath}`);
    await expect(
      page.getByText(
        "First assistant output — updated after the typed answer.",
        { exact: true },
      ),
    ).toBeVisible();
    await expect(
      page.getByText(
        "Second assistant utterance: the native fixture received the exact typed answers.",
        { exact: true },
      ),
    ).toBeVisible();
    await expect(
      page.getByText("typed native answer", { exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("radio", { name: /Stable.*Selected/i }),
    ).toBeChecked();
    const reentryMs = Math.round(performance.now() - reentryStart);
    test.info().annotations.push({
      type: "conversation-read-evidence",
      description: `initialReads=${initialReads};inputToFirstMs=${inputToFirstMs};answerToUpdateMs=${answerToUpdateMs};responseToVisibleUpperMs=${responseToVisibleUpperMs};reconnectMs=${reconnectMs};reentryMs=${reentryMs};regionReads=${regionReads}`,
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
