import { expect, test } from "@playwright/test";
import superjson from "superjson";
import {
  fixtureAgentAuthentication,
  fixtureCatalogue,
} from "./connectionFixture";

test("stale selection conflict refreshes the expected reference and permits an explicit new selection", async ({
  page,
}) => {
  let selections = 0;
  let reads = 0;
  const initial = fixtureAgentAuthentication("en").agent;
  const changed = {
    authenticationId: "other-credential",
    name: "Other tab selection",
    kind: "openaiApiKey" as const,
    requestId: "other-tab",
    selectedAt: 1,
  };
  await page.route("**/api/trpc/authentications.list", async (route) => {
    await route.fulfill({
      json: { result: { data: superjson.serialize(fixtureCatalogue.page) } },
    });
  });
  await page.route("**/api/trpc/computers.get", async (route) => {
    reads += 1;
    await route.fulfill({
      json: {
        result: {
          data: superjson.serialize({
            ...initial,
            authentication:
              selections > 1
                ? {
                    ...changed,
                    authenticationId: "key-fixture",
                    name: "Saved selection",
                  }
                : changed,
          }),
        },
      },
    });
  });
  await page.route("**/api/trpc/authentications.selectAgent", async (route) => {
    selections += 1;
    if (selections === 1) {
      await route.fulfill({
        status: 409,
        json: {
          error: {
            json: {
              message: "Conflict",
              code: -32009,
              data: {
                code: "CONFLICT",
                path: "authentications.selectAgent",
                reason: "DOMAIN_REASON_CONFLICT",
                httpStatus: 409,
              },
            },
          },
        },
      });
      return;
    }
    const body: unknown = route.request().postDataJSON();
    if (
      !body ||
      typeof body !== "object" ||
      !("json" in body) ||
      !body.json ||
      typeof body.json !== "object" ||
      !("expected" in body.json)
    ) {
      throw new Error("A fixture expected selection is required");
    }
    expect(body.json.expected).toEqual({
      authenticationId: changed.authenticationId,
      requestId: changed.requestId,
    });
    await route.fulfill({
      json: {
        result: {
          data: superjson.serialize({
            type: "selection",
            agentId: initial.id,
            authenticationId: "key-fixture",
          }),
        },
      },
    });
  });
  await page.goto(
    "/iframe.html?id=authentication-agent-selection-flow--current&viewMode=story",
  );
  await page.getByRole("button", { name: "Change", exact: true }).click();
  const card = page
    .getByRole("heading", { name: "개발용 API", exact: true })
    .locator("..");
  await card
    .getByRole("button", { name: "Choose authentication", exact: true })
    .click();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(
    page
      .locator("#storybook-root")
      .getByText("Other tab selection", { exact: true }),
  ).toBeVisible();
  await card
    .getByRole("button", { name: "Choose authentication", exact: true })
    .click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  expect(selections).toBe(2);
  expect(reads).toBeGreaterThan(0);
});
