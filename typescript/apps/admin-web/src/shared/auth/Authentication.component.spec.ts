import { expect, test } from "@playwright/test";

test("Admin consumers share one identity and explicitly recover initial infrastructure error", async ({
  page,
}) => {
  let requests = 0;
  await page.route("**/api/trpc/auth.identity**", (route) => {
    requests += 1;
    return route.fulfill({
      json: {
        result: {
          data: {
            json: {
              id: "fixture-admin",
              email: "recovered@example.com",
              isAdmin: true,
            },
          },
        },
      },
    });
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
    page.getByRole("status").filter({ hasText: "recovered@example.com" }),
  ).toHaveCount(2);
  expect(requests).toBe(1);
});

test("Admin logout uses the shared boundary and its own login mount", async ({
  page,
}) => {
  let requests = 0;
  await page.route("**/admin-probe/api/logout", async (route) => {
    requests += 1;
    expect(route.request().method()).toBe("POST");
    await route.fulfill({
      status: 303,
      headers: { location: "/admin-probe/login" },
    });
  });
  await page.route("**/admin-probe/login", (route) =>
    route.fulfill({ contentType: "text/html", body: "<h1>Admin login</h1>" }),
  );
  await page.goto(
    "/iframe.html?id=common-ui-authentication--authenticated&viewMode=story",
  );
  await expect(
    page.getByRole("status").filter({ hasText: "admin@example.com" }),
  ).toHaveCount(2);
  await page
    .getByRole("button", { name: "Log out header", exact: true })
    .click();
  await expect(page).toHaveURL(/\/admin-probe\/login$/);
  expect(requests).toBe(1);
});
