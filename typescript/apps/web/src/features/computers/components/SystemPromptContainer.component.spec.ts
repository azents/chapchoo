import { expect, test } from "@playwright/test";
import superjson from "superjson";
import { z } from "zod";
import { containerUser } from "../chat/components/chatContainerFixture";
import { systemPromptCopy } from "../systemPromptCopy";

const copy = systemPromptCopy("en");
test("system prompt container submits exact text and reads the saved setting after reopen", async ({
  page,
}) => {
  let saved = "Initial server setting";
  const submits: string[] = [];
  let reject = true;
  await page.route("**/api/trpc/**", async (route) => {
    const method = new URL(route.request().url()).pathname.split("/").at(-1);
    if (method === "auth.identity") {
      await route.fulfill({
        json: { result: { data: superjson.serialize(containerUser) } },
      });
    } else if (method === "computers.systemPrompt") {
      await route.fulfill({
        json: { result: { data: superjson.serialize({ content: saved }) } },
      });
    } else if (method === "computers.saveSystemPrompt") {
      const input = z
        .object({ json: z.object({ content: z.string() }) })
        .parse(route.request().postDataJSON()).json;
      submits.push(input.content);
      if (reject) {
        await route.fulfill({
          status: 500,
          json: {
            error: {
              json: {
                message: "Save failed",
                code: -32603,
                data: {
                  code: "INTERNAL_SERVER_ERROR",
                  path: method,
                  httpStatus: 500,
                },
              },
            },
          },
        });
      } else {
        saved = input.content;
        await route.fulfill({
          json: { result: { data: superjson.serialize({ content: saved }) } },
        });
      }
    } else {
      throw new Error(`Unexpected method ${method}`);
    }
  });
  await page.goto(
    "/iframe.html?id=computers-system-prompt-transport--editor&viewMode=story",
  );
  const open = page.getByRole("button", { name: copy.edit, exact: true });
  await open.click();
  const field = page.getByRole("textbox", { name: copy.field, exact: true });
  await expect(field).toHaveValue(saved);
  const exact = "  First instruction\n원문\n\n";
  await field.fill(exact);
  await page.getByRole("button", { name: copy.save, exact: true }).click();
  await expect(page.getByText(copy.saveFailed, { exact: true })).toBeVisible();
  await expect(field).toHaveValue(exact);
  expect(submits).toEqual([exact]);
  reject = false;
  await page.getByRole("button", { name: copy.save, exact: true }).click();
  await expect(field).toHaveCount(0);
  await open.click();
  await expect(field).toHaveValue(exact);
  await field.fill("");
  await page.getByRole("button", { name: copy.save, exact: true }).click();
  await expect(field).toHaveCount(0);
  expect(submits).toEqual([exact, exact, ""]);
});
