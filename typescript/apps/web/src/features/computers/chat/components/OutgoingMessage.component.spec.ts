import { expect, test } from "@playwright/test";

test("queue pagination exposes retained failures beyond the latest 24 after reload", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 4000 });
  await page.goto(
    "/iframe.html?id=chat-conversation--queue-pages&viewMode=story",
  );
  await expect(page.locator("[data-input-operation]")).toHaveCount(30);
  const oldest = page.locator('[data-input-operation="queue-page-0"]');
  await expect(oldest).toContainText("Queued item 0");
  await expect(
    oldest.getByRole("button", { name: "재시도", exact: true }),
  ).toBeEnabled();
  await expect(
    oldest.getByRole("button", { name: "삭제", exact: true }),
  ).toBeEnabled();
});

for (const width of [390, 1280]) {
  test(`queued and failed bubbles are dimmed without dimming recovery controls/${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    for (const state of ["queue-waiting", "queue-delivery-error"]) {
      await page.goto(
        `/iframe.html?id=chat-conversation--${state}&viewMode=story&globals=locale:ko-KR`,
      );
      const row = page.locator("[data-input-operation]").last();
      await expect(row).toBeVisible();
      await expect(row.locator("[data-user-bubble]")).toHaveCSS(
        "opacity",
        "0.6",
      );
      await expect(row).toHaveCSS("opacity", "1");
      await expect(
        page
          .locator("[data-user-bubble]")
          .filter({ hasText: "안녕, 지금 대화할 수 있어?" })
          .first(),
      ).toHaveCSS("opacity", "1");
      if (state === "queue-waiting") {
        await expect(row.getByRole("alert")).toHaveCount(0);
      } else {
        const retry = row.getByRole("button", { name: "재시도", exact: true });
        const remove = row.getByRole("button", { name: "삭제", exact: true });
        await expect(row.getByRole("alert")).toBeVisible();
        await expect(retry).toHaveText("");
        await expect(remove).toHaveText("");
        for (const button of [retry, remove]) {
          await expect(button).toBeEnabled();
          await expect(button).toHaveCSS("width", "28px");
          await expect(button).toHaveCSS("height", "28px");
          await button.focus();
          await expect(button).toBeFocused();
        }
        await retry.focus();
        await expect(
          page.getByRole("tooltip").filter({ hasText: "재시도" }),
        ).toBeVisible();
        await retry.hover();
        await retry.click();
        await expect(
          page.locator('[data-input-operation="queue-failed"]'),
        ).toHaveCount(0);
        await expect(
          page.locator(
            '[data-input-operation="queue-requeued"] [data-user-bubble]',
          ),
        ).toHaveCSS("opacity", "0.6");
      }
    }
  });

  test(`enqueue failure retains the composer draft without a pending bubble/${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(
      "/iframe.html?id=chat-conversation--composer-request-error&viewMode=story&globals=locale:ko-KR",
    );
    await expect(
      page.getByRole("textbox", { name: "메시지", exact: true }),
    ).toHaveValue("두 저장소의 작업 현황을 정리해줘.");
    await expect(page.getByRole("alert")).toContainText("전송 요청 실패");
    await expect(page.locator("[data-input-operation]")).toHaveCount(0);
  });
}
