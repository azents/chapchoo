import { expect, test } from "@playwright/test";

for (const story of [
  "computers-agents--running",
  "computers-agent-creation--ready",
]) {
  test(`${story} keeps workspace chrome outside the Agent theme`, async ({
    page,
  }) => {
    await page.goto(`/iframe.html?id=${story}&viewMode=story`);
    const logout = page
      .getByRole("banner")
      .getByRole("button", { name: "Log out", exact: true });
    await expect(logout).toBeVisible();
    const workspaceAccent = await page.evaluate(() =>
      getComputedStyle(document.documentElement)
        .getPropertyValue("--chapchoo-color-accent")
        .trim(),
    );
    await expect
      .poll(() =>
        logout.evaluate((element) =>
          getComputedStyle(element)
            .getPropertyValue("--chapchoo-color-accent")
            .trim(),
        ),
      )
      .toBe(workspaceAccent);
    const agentAction = page.getByRole("main").getByRole("button", {
      name: story.includes("--running") ? "Edit profile" : "Create Agent",
      exact: true,
    });
    await expect(agentAction).toBeVisible();
    const agentAccent = await agentAction.evaluate((element) =>
      getComputedStyle(element)
        .getPropertyValue("--chapchoo-color-accent")
        .trim(),
    );
    expect(agentAccent).not.toBe(workspaceAccent);
    if (story.includes("--running")) {
      await page.goto(
        "/iframe.html?id=computers-agents--stop-confirmation&viewMode=story",
      );
      const save = page
        .getByRole("dialog")
        .getByRole("button", { name: "Confirm", exact: true });
      await expect(save).toBeVisible();
      expect(
        await save.evaluate((element) =>
          getComputedStyle(element)
            .getPropertyValue("--chapchoo-color-accent")
            .trim(),
        ),
      ).toBe(agentAccent);
    }
  });
}
