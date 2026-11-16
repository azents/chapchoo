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
    await expect(input).toHaveText("");
    await expect(input).toBeFocused();
    await expect(input).toHaveAttribute("data-blurs", "0");
    await input.fill("계속 입력");
    await expect(input).toBeFocused();
  });
}

test("composer exposes the verified plain text editing surface with ordinary prediction hints", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--connected&viewMode=story",
  );
  const input = page.getByRole("textbox", { name: "메시지", exact: true });
  await expect(input).toHaveAttribute("contenteditable", "plaintext-only");
  await expect(input).toHaveAttribute("aria-multiline", "true");
  await expect(input).toHaveAttribute("inputmode", "text");
  await expect(input).toHaveAttribute("autocorrect", "on");
  await expect(input).toHaveAttribute("autocapitalize", "sentences");
  await expect(input).toHaveAttribute("spellcheck", "true");
  const id = await input.getAttribute("id");
  expect(id).toBeTruthy();
  await expect(input).toHaveAccessibleName("메시지");
  await expect(input).toHaveAttribute(
    "aria-placeholder",
    "코기 · 메시지 보내기",
  );
});

test("editor retains Korean composition, Unicode and line breaks without premature submission", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--connected&viewMode=story",
  );
  const input = page.getByRole("textbox", { name: "메시지", exact: true });
  await input.focus();
  await input.dispatchEvent("compositionstart", { data: "" });
  await input.fill("한글");
  await input.dispatchEvent("keydown", {
    key: "Enter",
    code: "Enter",
    keyCode: 229,
    isComposing: true,
  });
  await expect(input).toHaveText("한글");
  await input.dispatchEvent("compositionend", { data: "한글" });
  await input.press("End");
  await input.press("Shift+Enter");
  await input.pressSequentially("둘째 줄 🙂 <script>text</script>");
  expect(await input.innerText()).toBe(
    "한글\n둘째 줄 🙂 <script>text</script>",
  );
  await expect(input.locator("script")).toHaveCount(0);
  await input.press("Enter");
  await expect(input).toHaveText("");
  await expect(page.locator("[data-user-bubble]").last()).toContainText("한글");
  await expect(page.locator("[data-user-bubble]").last()).toContainText(
    "둘째 줄",
  );
});

test("paste inserts only original plain text at the selection", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--connected&viewMode=story",
  );
  const input = page.getByRole("textbox", { name: "메시지", exact: true });
  await input.fill("앞뒤");
  await input.press("Home");
  await input.press("ArrowRight");
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.evaluate(async () => {
    await navigator.clipboard.write([
      new ClipboardItem({
        "text/plain": new Blob(["<b>원문</b>\n🙂"], { type: "text/plain" }),
        "text/html": new Blob(
          ["<img src=x onerror=alert(1)><b>다른 내용</b>"],
          { type: "text/html" },
        ),
      }),
    ]);
  });
  await input.press("ControlOrMeta+v");
  expect(await input.innerText()).toBe("앞<b>원문</b>\n🙂뒤");
  await expect(input.locator("img, b")).toHaveCount(0);
  await expect(input).toBeFocused();
  await input.press("ControlOrMeta+z");
  expect(await input.innerText()).toBe("앞뒤");
});

test("native caret sentinels do not add draft text or hide the empty placeholder", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=chat-message-editor--empty&viewMode=story");
  const input = page.getByRole("textbox", { name: "메시지", exact: true });
  const draft = page.locator("[data-draft]");
  await input.fill("a");
  await input.press("End");
  await input.press("Shift+Enter");
  await expect(draft).toHaveText(JSON.stringify("a\n"));
  await input.press("Shift+Enter");
  await expect(draft).toHaveText(JSON.stringify("a\n\n"));
  await input.pressSequentially("b");
  await expect(draft).toHaveText(JSON.stringify("a\n\nb"));
  await input.press("ControlOrMeta+a");
  await input.press("Backspace");
  await expect(draft).toHaveText('""');
  await expect(input).toHaveAttribute("data-empty", "true");
  await input.fill("\n\n본문");
  await expect(draft).toHaveText(JSON.stringify("\n\n본문"));
  await input.fill("  본문\n\n");
  await expect(draft).toHaveText(JSON.stringify("  본문\n\n"));
  for (const value of [
    "\n\n본문",
    "a\n\nb\n",
    "한글 🙂\t  공백\u00a0끝",
    "\n\n",
  ]) {
    await input.fill(value);
    await expect(draft).toHaveText(JSON.stringify(value));
  }
});

test("acceptance during composition cannot restore the already submitted draft", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--connected&viewMode=story",
  );
  const input = page.getByRole("textbox", { name: "메시지", exact: true });
  await input.dispatchEvent("compositionstart", { data: "" });
  await input.fill("조합 중 전송");
  const send = page.getByRole("button", { name: "보내기", exact: true });
  await send.click();
  await expect(send).toBeDisabled();
  await input.dispatchEvent("compositionend", { data: "조합 중 전송" });
  await expect(input).toHaveText("");
  await expect(send).toBeDisabled();
  await expect(
    page
      .locator("[data-user-bubble]")
      .getByText("조합 중 전송", { exact: true }),
  ).toHaveCount(1);
  await input.pressSequentially("다음 초안");
  await expect(input).toHaveText("다음 초안");
});

test("long placeholder does not increase empty editor height", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const input = page.getByRole("textbox", { name: "메시지", exact: true });
  await page.goto("/iframe.html?id=chat-message-editor--empty&viewMode=story");
  const empty = await input.boundingBox();
  await page.goto(
    "/iframe.html?id=chat-message-editor--long-placeholder&viewMode=story",
  );
  const long = await input.boundingBox();
  expect(empty?.height).toBe(long?.height);
});
