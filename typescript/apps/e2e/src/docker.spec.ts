import { test, expect } from "@playwright/test";
import { Assembly } from "@chapchoo/testenv/assembly";
import type { Page } from "@playwright/test";

async function power(page: Page, value: string): Promise<void> {
  await expect(
    page
      .getByText("Computer power", { exact: true })
      .locator("..")
      .getByText(value, { exact: true }),
  ).toBeVisible({ timeout: 90_000 });
}
async function connected(page: Page): Promise<void> {
  await expect(
    page
      .getByText("Computer Connector", { exact: true })
      .locator("..")
      .getByText("Connected", { exact: true }),
  ).toBeVisible({ timeout: 90_000 });
}
async function confirm(page: Page, action: string): Promise<void> {
  await page.getByRole("button", { name: action, exact: true }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Confirm", exact: true })
    .click();
}

test("real Docker preserves the original saved data across native execution replacement and removes only owned resources", async ({
  browser,
}) => {
  test.setTimeout(420_000);
  // Required Linux path: a missing authorized daemon/image is a failure, not skip.
  const assembly = await Assembly.create(true);
  const context = await browser.newContext({
    locale: "en-US",
    timezoneId: "Asia/Seoul",
  });
  try {
    await assembly.initializeAdministrator();
    await assembly.startDockerProvider();
    const docker = assembly.docker;
    if (!docker) {
      throw new Error("Real Docker fixture was not initialized");
    }
    const owner = await context.newPage();
    await owner.goto(`${assembly.generalUrl}/login`);
    await owner
      .getByRole("textbox", { name: "Email address", exact: true })
      .fill(assembly.administrator.email);
    await owner
      .getByLabel(/^Password\s*\*?$/)
      .fill(assembly.administrator.password);
    await owner.getByRole("button", { name: "Log in", exact: true }).click();
    await expect(
      owner.getByRole("heading", { name: "Agents", exact: true }),
    ).toBeVisible();
    await owner
      .getByRole("link", { name: "Create Agent", exact: true })
      .click();
    await owner
      .getByRole("textbox", { name: "Agent name", exact: true })
      .fill("Native Docker companion");
    await owner
      .getByRole("combobox", { name: "Computer Provider", exact: true })
      .click();
    await owner
      .getByRole("option", { name: "Acceptance Docker", exact: true })
      .click({ timeout: 90_000 });
    await owner
      .getByRole("button", { name: "Create Agent", exact: true })
      .click();
    await expect(
      owner.getByRole("heading", {
        name: "Native Docker companion",
        exact: true,
      }),
    ).toBeVisible();
    await power(owner, "Running");
    await connected(owner);
    const original = await docker.current();
    expect(original.user).toBe("10001:10001");
    expect(original.noCopy).toBe(false);
    await docker.writeMarker(original);
    expect(await docker.readMarker(original)).toBe("saved-original-data");
    expect(await docker.canariesPresent()).toBe(true);

    await confirm(owner, "Stop");
    await power(owner, "Stopped");
    expect(await docker.containerAbsent(original.container)).toBe(true);
    await owner.getByRole("button", { name: "Start", exact: true }).click();
    await power(owner, "Running");
    await connected(owner);
    const successor = await docker.current();
    expect(successor.container).not.toBe(original.container);
    expect(successor.volume).toBe(original.volume);
    expect(successor.daemon).toBe(original.daemon);
    expect(successor.noCopy).toBe(true);
    expect(await docker.readMarker(successor)).toBe("saved-original-data");

    // Emptying saved contents is a test-owned write, not resource replacement.
    // Restart must keep that original empty volume with no image population.
    await docker.clearSaved(successor);
    await confirm(owner, "Restart");
    await expect
      .poll(async () => (await docker.successor(successor)) !== null, {
        timeout: 90_000,
      })
      .toBe(true);
    await power(owner, "Running");
    await connected(owner);
    const restarted = await docker.current();
    expect(restarted.container).not.toBe(successor.container);
    expect(restarted.volume).toBe(original.volume);
    expect(restarted.noCopy).toBe(true);
    expect(await docker.savedEmpty(restarted)).toBe(true);
    expect(await docker.canariesPresent()).toBe(true);
    await confirm(owner, "Delete Agent");
    await expect(
      owner.getByText("No Agents yet.", { exact: true }),
    ).toBeVisible({ timeout: 90_000 });
    await expect
      .poll(() => docker.productAbsent(), { timeout: 90_000 })
      .toBe(true);
    expect(await docker.canariesPresent()).toBe(true);
    // Explicit safe native evidence only: no bootstrap/environment/certificate dump.
    test.info().annotations.push({
      type: "native-docker",
      description: JSON.stringify({
        ...docker.evidence,
        original: original.container,
        successor: successor.container,
        restarted: restarted.container,
        volume: original.volume,
        cleanup: "confirmed",
        canaries: "unchanged",
        source: "explicit-unix",
        network: "bridge",
      }),
    });
  } finally {
    try {
      await context.close();
    } finally {
      await assembly.close();
    }
  }
});
