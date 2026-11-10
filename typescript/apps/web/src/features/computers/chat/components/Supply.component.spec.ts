import { expect, test } from "@playwright/test";

test("owner settings keep saved and observed applied choices separate", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-chat-supply--saved-differs-from-applied&viewMode=story",
  );
  await expect(
    page.getByText("/owner/selected codex", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("Observed applied selection", { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("alert")).toContainText(
    "not on the next message",
  );
  await page
    .getByRole("radio", { name: "Existing Codex installation", exact: true })
    .check();
  await page
    .getByRole("textbox", { name: "Codex executable path", exact: true })
    .fill("/owner/custom codex");
  await page
    .getByRole("button", { name: "Save selection", exact: true })
    .click();
  await expect(
    page.getByLabel("Fixture save count", { exact: true }),
  ).toHaveText("1");
  await expect(
    page.getByLabel("Fixture read count", { exact: true }),
  ).toHaveText("0");
});
test("uncertain settings retain a read-only recovery action without another save", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-chat-supply--uncertain-save&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: "Save selection", exact: true }),
  ).toBeDisabled();
  await expect(page.getByRole("radio").first()).toBeDisabled();
  await page
    .getByRole("button", { name: "Check saved selection", exact: true })
    .click();
  await expect(
    page.getByLabel("Fixture read count", { exact: true }),
  ).toHaveText("1");
  await expect(
    page.getByLabel("Fixture save count", { exact: true }),
  ).toHaveText("0");
});
test("an absent applied observation does not pretend the saved managed default is running", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-chat-supply--managed-default&viewMode=story",
  );
  await expect(
    page
      .getByRole("region", { name: "Observed applied selection", exact: true })
      .getByText("Not observed", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("radio", { name: "Managed preparation", exact: true }),
  ).toBeChecked();
});

test("exact cached preparation and changed authentication stay distinct from saved supply", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-chat-supply--saved-differs-from-applied&viewMode=story",
  );
  const preparation = page.getByRole("region", {
    name: "Applied preparation",
    exact: true,
  });
  await expect(preparation).toContainText("Cached after lookup failure");
  await expect(preparation).toContainText(
    "Node v24.0.0 · ACP 2.6.0 · Codex 0.165.0",
  );
  await expect(
    page.getByRole("region", { name: "Applied authentication", exact: true }),
  ).toContainText("differs from the running process");
});

test("historical unknown details never infer preparation or authentication from a managed default", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-chat-supply--managed-default&viewMode=story",
  );
  await expect(
    page.getByRole("region", { name: "Applied preparation", exact: true }),
  ).toHaveText("Applied preparationNot observed");
  await expect(
    page.getByRole("region", { name: "Applied authentication", exact: true }),
  ).toHaveText("Applied authenticationNot observed");
});
