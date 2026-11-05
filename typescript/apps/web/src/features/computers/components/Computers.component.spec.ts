import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { expect, test } from "@playwright/test";
import { creationCopy } from "../creationCopy";
import { computerDetailCopy } from "../detailCopy";
const copy = computerCopy("en-US");
const profile = creationCopy("en-US");
const detailCopy = computerDetailCopy("en-US");
test("Exact unresolved creation remains recoverable after its Provider disappears", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-creation--unresolved-creation-without-provider&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: copy.recoverCreation, exact: true }),
  ).toBeEnabled();
  await expect(
    page.getByRole("textbox", { name: copy.name, exact: true }),
  ).toBeDisabled();
  await expect(page.getByText(copy.lostSubmission)).toBeVisible();
  await expect(
    page.getByRole("textbox", { name: profile.handle, exact: true }),
  ).toBeDisabled();
  await expect(
    page
      .getByRole("group", { name: profile.image, exact: true })
      .getByRole("button", { name: "Corgi", exact: true }),
  ).toBeDisabled();
  await expect(
    page
      .getByRole("group", { name: profile.palette, exact: true })
      .getByRole("button", { name: profile.palettes.sage, exact: true }),
  ).toBeDisabled();
  const images = page
    .getByRole("group", { name: profile.image, exact: true })
    .getByRole("button");
  await expect(images).toHaveCount(6);
  for (const button of await images.all()) {
    await expect(button).toBeDisabled();
  }
  const palettes = page
    .getByRole("group", { name: profile.palette, exact: true })
    .getByRole("button");
  await expect(palettes).toHaveCount(7);
  for (const button of await palettes.all()) {
    await expect(button).toBeDisabled();
  }
});
test("creation keeps profile selections visible and blocks unverified handles", async ({
  page,
}) => {
  for (const state of ["checking-handle", "occupied-handle"]) {
    await page.goto(
      `/iframe.html?id=computers-agent-creation--${state}&viewMode=story`,
    );
    await expect(
      page.getByRole("button", { name: copy.create, exact: true }),
    ).toBeDisabled();
    await expect(
      page.getByRole("textbox", { name: profile.handle, exact: true }),
    ).toHaveValue("my-agent");
    await expect(
      page
        .getByRole("group", { name: profile.image, exact: true })
        .getByRole("button", { name: "Corgi", exact: true }),
    ).toHaveAttribute("aria-pressed", "true");
  }
  await expect(page.getByText(profile.occupied, { exact: true })).toBeVisible();
});
test("catalogue failure retains the chosen image and color without allowing a new create", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-creation--catalogue-unavailable&viewMode=story",
  );
  await expect(
    page
      .getByRole("group", { name: profile.image, exact: true })
      .getByRole("button", { name: "Corgi", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(
    page
      .getByRole("group", { name: profile.palette, exact: true })
      .getByRole("button", { name: profile.palettes.sage, exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(
    page.getByRole("button", { name: copy.create, exact: true }),
  ).toBeDisabled();
});
test("profile creation renders the real selection and permanent-handle guidance", async ({
  page,
}, info) => {
  await page.goto(
    "/iframe.html?id=computers-agent-creation--ready&viewMode=story",
  );
  await expect(
    page.getByText(profile.permanent, { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: copy.create, exact: true }),
  ).toBeEnabled();
  await expect(
    page
      .getByRole("region", { name: profile.preview, exact: true })
      .locator('img[src="/design-assets/03-corgi-216.webp"]'),
  ).toBeVisible();
  await page.screenshot({
    path: info.outputPath("creation-desktop.png"),
    fullPage: true,
  });
});
test("Korean profile creation keeps every field reachable on narrow screens", async ({
  page,
}, info) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto(
    "/iframe.html?id=computers-agent-creation--korean&viewMode=story",
  );
  const localized = creationCopy("ko-KR");
  const handle = page.getByRole("textbox", {
    name: localized.handle,
    exact: true,
  });
  await handle.focus();
  await expect(handle).toBeFocused();
  await expect(
    page.getByText(localized.permanent, { exact: true }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("group", { name: localized.image, exact: true })
      .getByRole("button", { name: "Corgi", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: info.outputPath("creation-mobile.png"),
    fullPage: true,
  });
});
test("Preparation retry is server-authorized and confirms preservation of the same Agent", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agents--retry-preparation&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: copy.retryPreparation, exact: true }),
  ).toBeEnabled();
  await page.goto(
    "/iframe.html?id=computers-agents--retry-preparation-confirmation&viewMode=story",
  );
  await expect(page.getByRole("dialog")).toContainText(
    copy.retryPreparationWarning,
  );
  await page.goto(
    "/iframe.html?id=computers-agents--stale-unknown&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: copy.retryPreparation, exact: true }),
  ).toHaveCount(0);
});
test("Signed-out Computer detail hides the prior identity and private Agent", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agents--signed-out&viewMode=story",
  );
  await expect(page.getByText(copy.accessUnavailable).first()).toBeVisible();
  await expect(
    page.getByText("Research companion", { exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByText("owner@example.invalid", { exact: true }),
  ).toHaveCount(0);
});

test("Computer read failure preserves the last-known detail and disables controls", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agents--stale-unknown&viewMode=story",
  );
  await expect(
    page.getByRole("heading", { name: "Research companion" }),
  ).toBeVisible();
  await expect(page.getByRole("status")).toContainText(copy.stale);
  await expect(
    page.getByRole("button", { name: copy.stop, exact: true }),
  ).toBeDisabled();
  await page.getByRole("button", { name: copy.details, exact: true }).click();
  const dialog = page.getByRole("dialog", { name: copy.details, exact: true });
  await expect(dialog.getByRole("status")).toContainText(copy.stale);
  await expect(
    dialog.getByText(copy.lastKnown, { exact: false }),
  ).toBeVisible();
  await expect(
    dialog.getByText(detailCopy.connection, { exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByText(detailCopy.management, { exact: true }),
  ).toBeVisible();
});
test("Creation Provider read failure retains the entered draft and recovery action", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-creation--provider-read-failure&viewMode=story",
  );
  await expect(
    page.getByRole("textbox", { name: copy.name, exact: true }),
  ).toHaveValue("Research companion");
  await expect(page.getByRole("alert")).toContainText(copy.unavailable);
  await expect(page.getByRole("button", { name: copy.create })).toBeDisabled();
});

test("Empty Agent list keeps its Team-qualified creation entry", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=computers-agents--empty&viewMode=story");
  await expect(
    page.getByRole("heading", { name: copy.agents, exact: true }),
  ).toBeVisible();
  await expect(page.getByText(copy.noAgents, { exact: true })).toBeVisible();
  await expect(
    page.getByRole("link", { name: profile.newAgent, exact: true }),
  ).toHaveAttribute("href", "/teams/my/new-agent");
});
test("saved profile cards lead with identity and stale reads have explicit recovery", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agents--stale-list&viewMode=story",
  );
  await expect(page.getByText("@research", { exact: true })).toBeVisible();
  await expect(
    page.locator('img[src="/design-assets/03-corgi-216.webp"]'),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Research companion", exact: true }),
  ).toHaveAttribute("href", "/teams/my/agents/research/chat");
  await expect(
    page.getByRole("button", { name: copy.retryRead, exact: true }),
  ).toBeEnabled();
  await expect(page.getByRole("status")).toContainText(copy.stale);
});
test("Computer detail and force warning fit narrow screens", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto(
    "/iframe.html?id=computers-agents--force-removal&viewMode=story",
  );
  await expect(page.getByRole("dialog")).toContainText(
    computerCopy("ko-KR").forceWarning,
  );
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
});
