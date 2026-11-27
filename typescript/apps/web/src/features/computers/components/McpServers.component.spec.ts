import { expect, test } from "@playwright/test";
import superjson from "superjson";
import { z } from "zod";
import { containerUser } from "../chat/components/chatContainerFixture";
import { mcpServersCopy } from "../mcpServersCopy";

const c = mcpServersCopy("en");
const story =
  "/iframe.html?id=computers-mcp-servers--settings-transport&viewMode=story";
const endpoint = "https://example.com/mcp";
const server = {
  id: "server",
  displayName: "Fixture",
  endpoint,
  hasBearer: true,
};
const success = (data: unknown): object => ({
  result: { data: superjson.serialize(data) },
});

test("late draft test cannot qualify an edit; lost saves recover by receipt without replay", async ({
  page,
}) => {
  const entered = Promise.withResolvers<void>();
  const release = Promise.withResolvers<void>();
  const complete = Promise.withResolvers<void>();
  let request = "";
  let changes = 0;
  let checks = 0;
  let tests = 0;
  let items: (typeof server)[] = [];
  await page.route("**/api/trpc/**", async (route) => {
    const method = new URL(route.request().url()).pathname.split("/").at(-1);
    const input: unknown = route.request().postDataJSON();
    switch (method) {
      case "auth.identity":
        await route.fulfill({ json: success(containerUser) });
        break;
      case "computers.integrations.list":
        await route.fulfill({ json: success({ items, nextAfter: null }) });
        break;
      case "computers.integrations.test": {
        tests += 1;
        if (tests === 1) {
          entered.resolve();
          await release.promise;
          await route.fulfill({ json: success({ type: "listed", count: 7 }) });
          complete.resolve();
        } else {
          await route.fulfill({ json: success({ type: "listed", count: 0 }) });
        }
        break;
      }
      case "computers.integrations.change": {
        changes += 1;
        const parsed = z
          .object({
            json: z.object({
              requestId: z.string(),
              change: z.object({ type: z.enum(["create", "delete"]) }),
            }),
          })
          .parse(input).json;
        if (parsed.change.type === "create") {
          request = parsed.requestId;
          await route.abort();
        } else {
          items = [];
          await route.fulfill({
            json: success({ id: server.id, action: "delete" }),
          });
        }
        break;
      }
      case "computers.integrations.result": {
        const parsed = z
          .object({ json: z.object({ requestId: z.string() }) })
          .parse(input).json;
        expect(parsed.requestId).toBe(request);
        checks += 1;
        if (checks === 1) {
          await route.fulfill({ json: success(null) });
        } else {
          items = [server];
          await route.fulfill({
            json: success({ id: server.id, action: "create" }),
          });
        }
        break;
      }
      default:
        throw new Error(`Unexpected method ${method}`);
    }
  });
  await page.goto(story);
  await page.getByRole("button", { name: `+ ${c.add}`, exact: true }).click();
  const modal = page.getByRole("dialog");
  await modal.getByLabel(c.name, { exact: true }).fill(server.displayName);
  await modal.getByLabel(c.endpoint, { exact: true }).fill(endpoint);
  await modal.getByLabel(c.auth, { exact: true }).selectOption("bearer");
  await modal
    .getByLabel(c.bearer, { exact: true })
    .fill("synthetic-fixture-bearer");
  await modal.getByRole("button", { name: c.test, exact: true }).click();
  await entered.promise;
  await modal
    .getByLabel(c.bearer, { exact: true })
    .fill("synthetic-fixture-updated");
  release.resolve();
  await complete.promise;
  await modal.getByRole("button", { name: c.test, exact: true }).click();
  await expect(modal.getByRole("status")).toHaveText(c.count(0));
  expect(changes).toBe(0);
  await modal.getByRole("button", { name: c.save, exact: true }).click();
  await expect(modal.getByText(c.uncertain)).toBeVisible();
  const check = modal.getByRole("button", { name: c.check, exact: true });
  await check.click();
  await expect(check).toBeEnabled();
  await expect(modal.getByText(c.uncertain)).toBeVisible();
  await modal.getByRole("button", { name: c.close, exact: true }).click();
  await expect(modal).toBeHidden();
  await page.reload();
  await expect(page.getByText(c.uncertain)).toBeVisible();
  await expect(
    page.getByRole("button", { name: `+ ${c.add}`, exact: true }),
  ).toBeDisabled();
  await page.getByRole("button", { name: c.check, exact: true }).click();
  await expect(page.getByText(c.uncertain)).toHaveCount(0);
  expect(changes).toBe(1);
  await page
    .getByRole("button", { name: `${c.edit}: ${server.displayName}` })
    .click();
  await expect(modal.getByLabel(c.bearer, { exact: true })).toHaveValue("");
  await modal.getByRole("button", { name: c.delete, exact: true }).click();
  await expect(modal.getByText(c.confirm)).toBeVisible();
  await modal.getByRole("button", { name: c.delete, exact: true }).click();
  await expect(page.getByText(c.empty)).toBeVisible();
  expect(changes).toBe(2);
});
