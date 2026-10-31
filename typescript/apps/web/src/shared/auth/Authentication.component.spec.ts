import { expect, test } from "@playwright/test";

test("shared consumers use SSR identity without mount fetch and recheck one cache", async ({
  page,
}) => {
  let requests = 0;
  await page.route("**/api/trpc/auth.identity**", async (route) => {
    requests += 1;
    await route.fulfill({
      json: {
        result: {
          data: {
            json: {
              id: "fixture-user",
              email: "updated@example.com",
              isAdmin: false,
            },
          },
        },
      },
    });
  });
  await page.goto(
    "/iframe.html?id=common-ui-authentication--authenticated&viewMode=story",
  );
  await expect(
    page.getByRole("status").filter({ hasText: "owner@example.com" }),
  ).toHaveCount(2);
  expect(requests).toBe(0);
  await page
    .getByRole("button", { name: "Recheck header", exact: true })
    .click();
  await expect(
    page.getByRole("status").filter({ hasText: "updated@example.com" }),
  ).toHaveCount(2);
  expect(requests).toBe(1);
  await page
    .getByRole("button", { name: "Clear content", exact: true })
    .click();
  await expect(
    page.getByRole("status").filter({ hasText: "SIGNED_OUT" }),
  ).toHaveCount(2);
});

test("transient identity failure stays an error and explicit recheck recovers", async ({
  page,
}) => {
  let requests = 0;
  await page.route("**/api/trpc/auth.identity**", async (route) => {
    requests += 1;
    if (requests === 1) {
      await route.fulfill({
        status: 503,
        json: {
          error: {
            json: {
              message: "Unavailable",
              code: -32603,
              data: {
                code: "INTERNAL_SERVER_ERROR",
                path: "auth.identity",
                reason: null,
                httpStatus: 503,
              },
            },
          },
        },
      });
    } else {
      await route.fulfill({
        json: {
          result: {
            data: {
              json: {
                id: "fixture-user",
                email: "recovered@example.com",
                isAdmin: false,
              },
            },
          },
        },
      });
    }
  });
  await page.goto(
    "/iframe.html?id=common-ui-authentication--unavailable&viewMode=story",
  );
  await expect(
    page.getByRole("status").filter({ hasText: "ERROR" }),
  ).toHaveCount(2);
  expect(requests).toBe(0);
  await page
    .getByRole("button", { name: "Recheck header", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Recheck header", exact: true }),
  ).toBeEnabled();
  expect(requests).toBe(1);
  expect(page.url()).toContain("iframe.html");
  await page
    .getByRole("button", { name: "Recheck content", exact: true })
    .click();
  await expect(
    page.getByRole("status").filter({ hasText: "recovered@example.com" }),
  ).toHaveCount(2);
  expect(requests).toBe(2);
});

test("identity loss is handled at common boundary without a page-specific effect", async ({
  page,
}) => {
  await page.route("**/api/trpc/auth.identity**", (route) =>
    route.fulfill({ json: { result: { data: { json: null } } } }),
  );
  await page.route("**/auth-probe/login", (route) =>
    route.fulfill({
      contentType: "text/html",
      body: "<h1>Login required</h1>",
    }),
  );
  await page.goto(
    "/iframe.html?id=common-ui-authentication--authenticated&viewMode=story",
  );
  await page
    .getByRole("button", { name: "Recheck header", exact: true })
    .click();
  await expect(page).toHaveURL(/\/auth-probe\/login$/);
});
