import { expect, test } from "@playwright/test";
import type { Locator } from "@playwright/test";

async function center(locator: Locator): Promise<{ x: number; y: number }> {
  return locator.evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    return { x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2 };
  });
}

test("navigation icon frames align and banner leaves an independent close target", async ({
  page,
}) => {
  await page.setViewportSize({ width: 402, height: 874 });
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--mobile-picker&viewMode=story",
  );
  await page.evaluate(() => document.fonts.ready);
  const drawer = page.locator(".mantine-Drawer-content");
  const trigger = drawer.getByRole("button", {
    name: "Switch workspace",
    exact: true,
  });
  const triggerCenter = await center(trigger);
  for (const icon of await trigger.locator("svg").all()) {
    expect(Math.abs((await center(icon)).y - triggerCenter.y)).toBeLessThan(1);
  }
  const close = drawer.getByRole("button", { name: "Close", exact: true });
  const triggerBounds = await trigger.boundingBox();
  const closeBounds = await close.boundingBox();
  expect(triggerBounds).not.toBeNull();
  expect(closeBounds?.width).toBeGreaterThanOrEqual(44);
  expect(closeBounds?.height).toBeGreaterThanOrEqual(44);
  if (triggerBounds && closeBounds) {
    expect(
      closeBounds.x - (triggerBounds.x + triggerBounds.width),
    ).toBeGreaterThanOrEqual(16);
  }
});

test("long workspace names retain bounded banners and balanced navigation padding", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 740 });
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--mobile-long-name&viewMode=story",
  );
  const drawer = page.locator(".mantine-Drawer-content");
  const trigger = drawer.getByRole("button", {
    name: "Switch workspace",
    exact: true,
  });
  const close = drawer.getByRole("button", { name: "Close", exact: true });
  await expect(close).toBeInViewport();
  const gap = await trigger.evaluate(
    (element, closeX) => closeX - element.getBoundingClientRect().right,
    (await close.boundingBox())?.x ?? 0,
  );
  expect(gap).toBeGreaterThanOrEqual(16);
  const selected = drawer.getByRole("link", { name: "Agents", exact: true });
  await expect(selected).toHaveCSS("padding-left", "14px");
  await expect(selected).toHaveCSS("padding-right", "14px");
  expect(
    await selected.evaluate(
      (element) => element.getBoundingClientRect().height,
    ),
  ).toBeGreaterThanOrEqual(44);
});

test("creation dialog owns explicit cancel, pending admission and uncertain-outcome feedback", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--creation&viewMode=story",
  );
  const dialog = page.getByRole("dialog", { name: "Create team", exact: true });
  await expect(
    dialog.getByRole("textbox", { name: "Team name", exact: true }),
  ).toHaveValue("Design team");
  await expect(
    dialog.getByRole("textbox", { name: "Handle", exact: true }),
  ).toHaveValue("design");
  await expect(
    dialog.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeEnabled();
  await expect(
    dialog.getByRole("button", { name: "Create", exact: true }),
  ).toBeEnabled();
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--creation-pending&viewMode=story",
  );
  await expect(
    dialog.getByRole("textbox", { name: "Team name", exact: true }),
  ).toBeDisabled();
  await expect(
    dialog.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeDisabled();
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--creation-error&viewMode=story",
  );
  await expect(dialog.getByRole("alert")).toContainText(
    "Check the team list before submitting again.",
  );
});

test("mobile picker overlays unchanged navigation with the existing companion", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--mobile-picker&viewMode=story",
  );
  const dialog = page.locator(".mantine-Drawer-content");
  await expect(
    dialog.getByRole("link", { name: "Your space", exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByRole("link", { name: "Design team", exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByRole("button", { name: "Create team", exact: true }),
  ).toBeVisible();
  await expect(dialog.getByRole("navigation")).toContainText("Agents");
  await expect(dialog.getByRole("navigation")).toContainText("Invitations");
  await expect(
    dialog.getByRole("button", {
      name: "Replay character animation",
      exact: true,
    }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});

test.describe("touch workspace overlay", () => {
  test.use({
    hasTouch: true,
    isMobile: true,
    viewport: { width: 402, height: 874 },
  });
  test("touch close is neutral, nested Escape preserves Drawer and close remains reachable", async ({
    page,
  }) => {
    await page.goto(
      "/iframe.html?id=navigation-workspace-chrome--mobile-interactive&viewMode=story",
    );
    const open = page.getByRole("button", {
      name: "Open navigation",
      exact: true,
    });
    await open.tap();
    const drawer = page.locator(".mantine-Drawer-content");
    const trigger = drawer.getByRole("button", {
      name: "Switch workspace",
      exact: true,
    });
    const navigation = drawer.getByRole("navigation");
    const before = await navigation.innerText();
    await trigger.tap();
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
    await expect(
      drawer.getByRole("link", { name: "Design team", exact: true }),
    ).toBeVisible();
    expect(await navigation.innerText()).toBe(before);
    await trigger.tap();
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await expect(drawer).toBeVisible();
    await trigger.tap();
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
    await expect(
      drawer.getByRole("link", { name: "Your space", exact: true }),
    ).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await expect(drawer).toBeVisible();
    await expect(trigger).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
    await expect(trigger).toBeFocused();
    await drawer.getByRole("button", { name: "Close", exact: true }).tap();
    await expect(drawer).not.toBeVisible();
    await expect(open).toBeFocused();
  });
  test("team creation preserves the Drawer and returns focus on cancel", async ({
    page,
  }) => {
    await page.goto(
      "/iframe.html?id=navigation-workspace-chrome--mobile-interactive&viewMode=story",
    );
    await page
      .getByRole("button", { name: "Open navigation", exact: true })
      .tap();
    const drawer = page.locator(".mantine-Drawer-content");
    const trigger = drawer.getByRole("button", {
      name: "Switch workspace",
      exact: true,
    });
    await trigger.tap();
    await drawer
      .getByRole("button", { name: "Create team", exact: true })
      .tap();
    const creation = page.getByRole("dialog", {
      name: "Create team",
      exact: true,
    });
    await expect(creation).toBeVisible();
    await expect(drawer).toBeVisible();
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await creation.getByRole("button", { name: "Cancel", exact: true }).tap();
    await expect(creation).not.toBeVisible();
    await expect(drawer).toBeVisible();
    await expect(trigger).toBeFocused();
    await trigger.tap();
    await drawer
      .getByRole("button", { name: "Create team", exact: true })
      .tap();
    await expect(
      creation.getByRole("textbox", { name: "Team name", exact: true }),
    ).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(creation).not.toBeVisible();
    await expect(drawer).toBeVisible();
    await expect(trigger).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(drawer).not.toBeVisible();
  });
});

test("resizing an open Drawer leaves one current trigger for desktop dialog focus", async ({
  page,
}) => {
  await page.setViewportSize({ width: 402, height: 874 });
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--responsive-interactive&viewMode=story",
  );
  await page
    .getByRole("button", { name: "Open navigation", exact: true })
    .click();
  const drawer = page.locator(".mantine-Drawer-content");
  await expect(drawer).toBeVisible();
  await page.setViewportSize({ width: 1280, height: 900 });
  const trigger = page.locator("aside").getByRole("button", {
    name: "Switch workspace",
    exact: true,
  });
  await expect(drawer).not.toBeVisible();
  await expect(
    page.getByRole("button", { name: "Switch workspace", exact: true }),
  ).toHaveCount(1);
  await trigger.click();
  await page.getByRole("button", { name: "Create team", exact: true }).click();
  const creation = page.getByRole("dialog", {
    name: "Create team",
    exact: true,
  });
  await expect(
    creation.getByRole("textbox", { name: "Team name", exact: true }),
  ).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(creation).not.toBeVisible();
  await expect(trigger).toBeFocused();
});

test("desktop team creation returns focus to the workspace banner on cancel and Escape", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--desktop-interactive&viewMode=story",
  );
  const trigger = page.locator("aside").getByRole("button", {
    name: "Switch workspace",
    exact: true,
  });
  const creation = page.getByRole("dialog", {
    name: "Create team",
    exact: true,
  });
  for (const dismissal of ["cancel", "escape"]) {
    await trigger.click();
    await page
      .getByRole("button", { name: "Create team", exact: true })
      .click();
    await expect(
      creation.getByRole("textbox", { name: "Team name", exact: true }),
    ).toBeFocused();
    if (dismissal === "cancel") {
      await creation
        .getByRole("button", { name: "Cancel", exact: true })
        .click();
    } else {
      await page.keyboard.press("Escape");
    }
    await expect(creation).not.toBeVisible();
    await expect(trigger).toBeFocused();
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
  }
});

test("desktop workspace banner toggles the picker with mouse and keyboard", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--desktop-interactive&viewMode=story",
  );
  const trigger = page.locator("aside").getByRole("button", {
    name: "Switch workspace",
    exact: true,
  });
  await trigger.click();
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  await trigger.click();
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
  await trigger.focus();
  await page.keyboard.press("Enter");
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  await expect(
    page.getByRole("link", { name: "Your space", exact: true }),
  ).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
  await expect(trigger).toBeFocused();
});

test("Admin chrome keeps Administration selected and shares the General logo destination", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=navigation-workspace-chrome--administration&viewMode=story",
  );
  await expect(
    page
      .getByRole("banner")
      .getByRole("link", { name: "chapchoo", exact: true }),
  ).toHaveAttribute("href", "/");
  await expect(
    page.getByRole("banner").getByText("admin", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Administration", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.getByRole("link", { name: "Your space", exact: true }),
  ).not.toHaveAttribute("aria-current", "page");
  await expect(
    page
      .locator("aside")
      .getByRole("link", { name: "Server settings", exact: true }),
  ).toBeVisible();
});
