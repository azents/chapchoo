import { expect, test } from "@playwright/test";
import superjson from "superjson";
import { z } from "zod";
import { instructionEvidenceCopy } from "../instructionEvidenceCopy";
import { containerUser } from "./chatContainerFixture";

const copy = instructionEvidenceCopy("en");
const inputSchema = z.object({
  json: z.object({
    agentId: z.literal("agent"),
    conversationId: z.literal("conversation"),
    beforeOperationId: z.string().optional(),
    operationId: z.string().optional(),
  }),
});
function item(operationId: string): {
  operationId: string;
  executionId: string;
  createdAt: number;
  appliedAt: number;
} {
  return {
    operationId,
    executionId: "same-computer-execution",
    createdAt: 1000,
    appliedAt: 1001,
  };
}
test("older history failure preserves selected originals and retries the same cursor", async ({
  page,
}) => {
  const cursors: (string | null)[] = [];
  const evidenceReads: string[] = [];
  let reject = true;
  await page.route("**/api/trpc/**", async (route) => {
    const method = new URL(route.request().url()).pathname.split("/").at(-1);
    if (method === "auth.identity") {
      await route.fulfill({
        json: { result: { data: superjson.serialize(containerUser) } },
      });
      return;
    }
    const input = inputSchema.parse(route.request().postDataJSON()).json;
    if (method === "conversations.instructionHistory") {
      cursors.push(input.beforeOperationId ?? null);
      if (input.beforeOperationId === "older" && reject) {
        await route.fulfill({
          status: 500,
          json: {
            error: {
              json: {
                message: "History read failed",
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
        return;
      }
      const data =
        input.beforeOperationId === void 0
          ? { items: [item("latest")], nextBefore: "latest" }
          : input.beforeOperationId === "latest"
            ? { items: [item("older")], nextBefore: "older" }
            : { items: [item("oldest")] };
      await route.fulfill({
        json: { result: { data: superjson.serialize(data) } },
      });
    } else if (
      method === "conversations.instructionEvidence" &&
      input.operationId
    ) {
      evidenceReads.push(input.operationId);
      await route.fulfill({
        json: {
          result: {
            data: superjson.serialize({
              ...item(input.operationId),
              agentId: input.agentId,
              conversationId: input.conversationId,
              sources: [
                {
                  role: "owner",
                  text: `  Original ${input.operationId}\n한국어 원문\n`,
                },
              ],
              document: `Original document ${input.operationId}`,
            }),
          },
        },
      });
    } else {
      throw new Error(`Unexpected method ${method}`);
    }
  });
  await page.goto(
    "/iframe.html?id=chat-prompt-sources-transport--history&viewMode=story",
  );
  await page.getByRole("button", { name: copy.title, exact: true }).click();
  const selection = page.getByRole("combobox", {
    name: copy.execution,
    exact: true,
  });
  await expect(selection).toHaveValue(/same-computer-execution.*latest/);
  await expect(
    page.getByText("Original latest", { exact: false }),
  ).toBeVisible();
  const more = page.getByRole("button", { name: copy.more, exact: true });
  await more.click();
  await selection.click();
  await page
    .getByRole("option", { name: /same-computer-execution.*older/ })
    .click();
  await expect(
    page.getByText("Original older", { exact: false }),
  ).toBeVisible();
  await more.click();
  await expect(page.getByRole("alert")).toHaveText(copy.moreError);
  await expect(selection).toHaveValue(/same-computer-execution.*older/);
  await expect(
    page.getByText("Original older", { exact: false }),
  ).toBeVisible();
  reject = false;
  await more.click();
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(more).toHaveCount(0);
  await expect(selection).toHaveValue(/same-computer-execution.*older/);
  await expect(
    page.getByText("Original older", { exact: false }),
  ).toBeVisible();
  await selection.click();
  await expect(page.getByRole("option")).toHaveCount(3);
  await page
    .getByRole("option", { name: /same-computer-execution.*oldest/ })
    .click();
  await expect(
    page.getByText("Original oldest", { exact: false }),
  ).toBeVisible();
  expect(cursors).toEqual([null, "latest", "older", "older"]);
  expect(evidenceReads).toEqual(["latest", "older", "oldest"]);
});
