import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { expect, test } from "@playwright/test";
import { creationCopy } from "../creationCopy";
const copy = computerCopy("en-US");
const profile = creationCopy("en-US");
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
    page.getByRole("combobox", { name: profile.image, exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("combobox", { name: profile.palette, exact: true }),
  ).toBeDisabled();
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
      page.getByRole("combobox", { name: profile.image, exact: true }),
    ).toHaveValue("Corgi");
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
    page.getByRole("combobox", { name: profile.image, exact: true }),
  ).toHaveValue("Corgi");
  await expect(
    page.getByRole("combobox", { name: profile.palette, exact: true }),
  ).toHaveValue("Sage");
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
    page.locator('img[src="/design-assets/03-corgi-216.webp"]'),
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
    page.getByRole("combobox", { name: localized.image, exact: true }),
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
  await expect(page.getByText(copy.lastKnown, { exact: false })).toBeVisible();
  await expect(
    page.getByRole("button", { name: copy.stop, exact: true }),
  ).toBeDisabled();
  await expect(page.getByText(copy.connector, { exact: true })).toBeVisible();
  await expect(page.getByText(copy.management, { exact: true })).toBeVisible();
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
    page.getByRole("link", { name: copy.create, exact: true }),
  ).toHaveAttribute("href", "/agents/new?team=team");
});
test("Agent identity is primary and stale list reads have explicit recovery", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agents--stale-list&viewMode=story",
  );
  await expect(
    page.getByRole("columnheader", { name: copy.name, exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("columnheader", { name: copy.computerStatus, exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Research companion", exact: true }),
  ).toHaveAttribute("href", "/agents/agent?team=team");
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
