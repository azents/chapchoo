import { expect, test } from "@playwright/test";

test.use({ hasTouch: true });
for (const activation of ["mouse", "touch", "enter"] as const) {
  test(`sending preserves composer focus without a blur/${activation}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(
      "/iframe.html?id=chat-conversation--connected&viewMode=story",
    );
    const input = page.getByRole("textbox", { name: "메시지", exact: true });
    await input.fill(`포커스 유지 ${activation}`);
    await input.evaluate((node) => {
      node.setAttribute("data-blurs", "0");
      node.addEventListener("blur", () =>
        node.setAttribute(
          "data-blurs",
          String(Number(node.getAttribute("data-blurs")) + 1),
        ),
      );
    });
    const send = page.getByRole("button", { name: "보내기", exact: true });
    if (activation === "mouse") {
      await send.click();
    } else if (activation === "touch") {
      await send.tap();
    } else {
      await input.press("Enter");
    }
    await expect(
      page.getByText(`포커스 유지 ${activation}`, { exact: true }),
    ).toHaveCount(1);
    await expect(input).toHaveValue("");
    await expect(input).toBeFocused();
    await expect(input).toHaveAttribute("data-blurs", "0");
    await input.fill("계속 입력");
    await expect(input).toBeFocused();
  });
}

test("composer exposes ordinary text-entry semantics matching Azents", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--connected&viewMode=story",
  );
  const input = page.getByRole("textbox", { name: "메시지", exact: true });
  await expect(input).toHaveAttribute("name", "chat-message");
  await expect(input).toHaveAttribute("inputmode", "text");
  await expect(input).toHaveAttribute("autocorrect", "on");
  await expect(input).toHaveAttribute("autocapitalize", "sentences");
  await expect(input).toHaveAttribute("spellcheck", "true");
  const id = await input.getAttribute("id");
  expect(id).toBeTruthy();
  expect(
    await input.evaluate((node) =>
      node instanceof HTMLTextAreaElement ? node.labels.length : 0,
    ),
  ).toBe(1);
  await expect(input).toHaveAttribute("placeholder", "코기 · 메시지 보내기");
});
