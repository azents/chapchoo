import { expect, test } from "@playwright/test";
import superjson from "superjson";
import { fixtureCatalogue } from "./connectionFixture";

for (const scenario of [
  { operation: "rename", action: "Rename", key: false, removed: false },
  {
    operation: "replaceKey",
    action: "Replace key",
    key: true,
    removed: false,
  },
  {
    operation: "delete",
    action: "Delete connection",
    key: false,
    removed: false,
  },
  { operation: "startDevice", action: "Reconnect", key: false, removed: false },
  { operation: "rename", action: "Rename", key: false, removed: true },
]) {
  test(`rejected ${scenario.operation} ${scenario.removed ? "removed target" : "version conflict"} closes stale management and refreshes the next deliberate action`, async ({
    page,
  }) => {
    const initial = fixtureCatalogue.page.items[scenario.key ? 1 : 0];
    if (!initial) {
      throw new Error("A management fixture target is required");
    }
    let rejected = false;
    let writes = 0;
    let refreshes = 0;
    const changed = {
      ...initial,
      currentRequestId: "other-tab-version",
      name: "Updated by another tab",
    };
    await page.route("**/api/trpc/authentications.list", async (route) => {
      if (rejected) {
        refreshes++;
      }
      await route.fulfill({
        json: {
          result: {
            data: superjson.serialize({
              items: rejected ? (scenario.removed ? [] : [changed]) : [initial],
              nextAfter: null,
            }),
          },
        },
      });
    });
    await page.route("**/api/trpc/authentications.*", async (route) => {
      const method = new URL(route.request().url()).pathname.split(".").at(-1);
      if (method === "list") {
        await route.fallback();
        return;
      }
      writes++;
      const body: unknown = route.request().postDataJSON();
      if (
        !body ||
        typeof body !== "object" ||
        !("json" in body) ||
        !body.json ||
        typeof body.json !== "object"
      ) {
        throw new Error("A management mutation input is required");
      }
      if (!rejected) {
        expect(method).toBe(scenario.operation);
        rejected = true;
        await route.fulfill({
          status: scenario.removed ? 404 : 412,
          json: {
            error: {
              json: {
                message: "Rejected",
                code: scenario.removed ? -32004 : -32012,
                data: {
                  code: scenario.removed ? "NOT_FOUND" : "PRECONDITION_FAILED",
                  path: `authentications.${method}`,
                  reason: scenario.removed
                    ? "DOMAIN_REASON_NOT_FOUND"
                    : "DOMAIN_REASON_INVALID_TRANSITION",
                  httpStatus: scenario.removed ? 404 : 412,
                },
              },
            },
          },
        });
        return;
      }
      expect(method).toBe("rename");
      expect(
        "expectedRequestId" in body.json && body.json.expectedRequestId,
      ).toBe(changed.currentRequestId);
      await route.fulfill({
        json: {
          result: {
            data: superjson.serialize({
              type: "record",
              authenticationId: changed.id,
            }),
          },
        },
      });
    });
    await page.goto(
      "/iframe.html?id=authentication-flow--management&viewMode=story",
    );
    const card = page
      .getByRole("heading", { name: initial.name })
      .locator("..");
    await card.getByRole("button", { name: "Manage", exact: true }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: scenario.action, exact: true })
      .click();
    if (scenario.operation === "replaceKey") {
      await page
        .getByLabel(/^API key\s*\*?$/)
        .fill("disposable-replacement-key");
    } else if (
      scenario.operation !== "delete" &&
      scenario.operation !== "startDevice"
    ) {
      await page.getByRole("textbox", { name: "Alias" }).fill("Draft");
    }
    if (scenario.operation !== "startDevice") {
      await page
        .getByRole("dialog")
        .getByRole("button", {
          name: scenario.operation === "delete" ? "Delete connection" : "Save",
          exact: true,
        })
        .click();
    }
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.getByRole("status")).toHaveText("Closed: 1");
    expect(writes).toBe(1);
    expect(refreshes).toBeGreaterThan(0);
    if (scenario.removed) {
      await expect(
        page.getByRole("heading", { name: initial.name }),
      ).toHaveCount(0);
      return;
    }
    const refreshed = page
      .getByRole("heading", { name: changed.name })
      .locator("..");
    await refreshed
      .getByRole("button", { name: "Manage", exact: true })
      .click();
    await page.getByRole("button", { name: "Rename", exact: true }).click();
    await page.getByRole("textbox", { name: "Alias" }).fill("New action");
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.getByRole("status")).toHaveText("Closed: 2");
    expect(writes).toBe(2);
  });
}
