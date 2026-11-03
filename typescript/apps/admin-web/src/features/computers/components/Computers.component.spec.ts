import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { expect, test } from "@playwright/test";
const copy = computerCopy("en-US");
test("Connected Providers expose independent read-only creation capabilities", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-operatorinventory--providers&viewMode=story",
  );
  await expect(page.getByText(copy.connected, { exact: true })).toHaveCount(2);
  await expect(page.getByText(copy.canCreate, { exact: true })).toBeVisible();
  await expect(
    page.getByText(copy.cannotCreate, { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: copy.create, exact: true }),
  ).toHaveCount(0);
});
test("Signed-out resource inventory hides prior resource identities", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-resources--signed-out&viewMode=story",
  );
  await expect(page.getByText(copy.accessUnavailable).first()).toBeVisible();
  await expect(
    page.getByText("UID: original-pvc-uid", { exact: true }),
  ).toHaveCount(0);
});
test("Resource diagnostic keeps original identity separate from unknown presence", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-resources--original-uid-unknown-presence&viewMode=story",
  );
  await expect(
    page.getByText("UID: original-pvc-uid", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("Presence: Unknown", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("Resource name: original-saved-environment", {
      exact: true,
    }),
  ).toBeVisible();
});
test("Operator inventory remains read-only and exposes unresolved cleanup separately", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-operatorinventory--residual-computer&viewMode=story",
  );
  await expect(page.getByRole("main").getByText(copy.readOnly)).toBeVisible();
  await expect(page.getByText(copy.residual)).toBeVisible();
  await expect(page.getByText(copy.power, { exact: true })).toBeVisible();
  await expect(page.getByText(copy.connector, { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: copy.force })).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: copy.stop, exact: true }),
  ).toHaveCount(0);
});
test("Docker diagnostics retain native identities and restorable cursors on a narrow viewport", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=computers-resources--docker-original-volume&viewMode=story",
  );
  await expect(
    page.getByText(copy.dockerVolume, { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText(`${copy.containerId}: ${"b".repeat(64)}`, { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText(`${copy.volumeCreatedAt}: 2026-11-03T06:45:15Z`, {
      exact: true,
    }),
  ).toBeVisible();
  await expect(page.getByText(/^Namespace:|^UID:/)).toHaveCount(0);
  const next = page.getByRole("link", { name: copy.next, exact: true });
  const href = await next.getAttribute("href");
  expect(href).not.toBeNull();
  const query = new URL(href ?? "", "http://localhost").searchParams;
  expect(query.get("nativeScope")).toBe(`daemon-${"a".repeat(300)}`);
  expect(query.get("nativeScopeKind")).toBe("DOCKER_DAEMON");
  await next.focus();
  await expect(next).toBeFocused();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
test("Disconnected inventory is retained on browser read failure", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-operatorinventory--stale&viewMode=story",
  );
  await expect(
    page.getByText("computer-retained", { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("status")).toContainText(copy.stale);
});
