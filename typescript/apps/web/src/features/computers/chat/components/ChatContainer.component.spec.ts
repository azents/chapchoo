import { expect, test } from "@playwright/test";
import superjson from "superjson";
import { containerInput, containerUser } from "./chatContainerFixture";

test("actual Chat container retains a failed enqueue draft without optimistic Chat or queue rows", async ({
  page,
}) => {
  const sends: unknown[] = [];
  const unexpected: string[] = [];
  await page.route("**/api/trpc/**", async (route) => {
    const method = new URL(route.request().url()).pathname.split("/").at(-1);
    if (method === "conversations.send") {
      sends.push(route.request().postDataJSON());
      await route.abort("failed");
      return;
    }
    let value: unknown;
    switch (method) {
      case "auth.identity":
        value = containerUser;
        break;
      case "computers.get":
        if (containerInput.initial.type !== "detail") {
          throw new Error("Fixture requires an Agent detail");
        }
        value = containerInput.initial.agent;
        break;
      case "conversations.region":
      case "conversations.activity":
        value = containerInput.initialConversation;
        break;
      case "conversations.messages":
        value = containerInput.initialMessages;
        break;
      default:
        unexpected.push(method ?? route.request().url());
        await route.abort();
        return;
    }
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ result: { data: superjson.serialize(value) } }),
    });
  });
  await page.goto(
    "/iframe.html?id=chat-container-transport--enqueue-failure&viewMode=story",
  );
  const input = page.getByRole("textbox", { name: "Message", exact: true });
  await expect(input).toBeEnabled();
  const draft = "Keep this exact failed draft 🙂";
  await input.fill(draft);
  await page.getByRole("button", { name: "Send", exact: true }).click();
  await expect(page.locator("#chat-input-error")).toBeVisible();
  await expect(input).toHaveValue(draft);
  await expect(page.locator("[data-input-operation]")).toHaveCount(0);
  await expect(page.locator("[data-user-bubble]")).toHaveCount(0);
  expect(sends).toHaveLength(1);
  expect(unexpected).toEqual([]);
});
