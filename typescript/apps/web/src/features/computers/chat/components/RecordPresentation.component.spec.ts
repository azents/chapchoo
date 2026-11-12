import { expect, test } from "@playwright/test";

for (const width of [390, 960]) {
  test(`approved full records preserve message meaning, originals and scoped primary at ${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    const colors: string[] = [];
    for (const theme of ["default", "scoped-teal"]) {
      await page.goto(
        `/iframe.html?id=chat-record-presentation--${theme}&viewMode=story`,
      );
      await page.evaluate(async () => {
        await document.fonts.ready;
      });
      const user = page.locator('[data-record-origin="user"]');
      const publication = page.locator('[data-record-origin="publication"]');
      const outputs = page.locator('[data-record-origin="assistant"]');
      await expect(user).toHaveCount(1);
      await expect(publication).toHaveCount(1);
      await expect(outputs).toHaveCount(2);
      await expect(
        page.getByText("Fixture Agent", { exact: true }),
      ).toHaveCount(0);
      await expect(user.getByText("나", { exact: true })).toHaveCount(0);
      await expect(publication.getByText("Agent", { exact: true })).toHaveCount(
        0,
      );
      await expect(
        user.getByRole("img", { name: "사람", exact: true }),
      ).toBeVisible();
      await expect(
        publication.getByRole("img", { name: "Agent", exact: true }),
      ).toBeVisible();
      await expect(
        outputs.first().getByText("모델 출력", { exact: true }),
      ).toBeVisible();
      await expect(user.getByRole("button").first()).toContainText(
        "채팅의 색상 설정을 확인하고",
      );
      await expect(publication.getByRole("button").first()).toContainText(
        "테마의 색상 변수를 정리했습니다.",
      );
      await expect(publication.getByRole("button").first()).not.toContainText(
        "mcp.chapchoo",
      );
      colors.push(
        await publication.evaluate(
          (element) => getComputedStyle(element).borderInlineStartColor,
        ),
      );
      if (theme === "scoped-teal") {
        await expect(publication).toHaveAttribute(
          "style",
          /--mantine-color-agent-teal-filled/,
        );
      }
      for (const row of [user, outputs.first(), outputs.last(), publication]) {
        await row.getByRole("button", { expanded: false }).first().click();
        await expect(row.getByRole("button").first()).toHaveAttribute(
          "aria-expanded",
          "true",
        );
      }
      await expect(
        publication.getByText("Message accepted.", { exact: true }),
      ).toBeVisible();
      await publication
        .getByRole("button", { name: "메시지 입력과 결과", exact: true })
        .click();
      await publication
        .locator("summary")
        .filter({ hasText: "도구 입력" })
        .click();
      await expect(
        publication
          .locator("pre")
          .filter({ hasText: /"server"\s*:\s*"chapchoo"/ }),
      ).toBeVisible();
      await expect(
        publication
          .locator("pre")
          .filter({ hasText: /"tool"\s*:\s*"publish_text"/ }),
      ).toBeVisible();
      await expect(
        publication.locator("pre").filter({ hasText: "fixture-message" }),
      ).toBeVisible();
      await expect(
        page.getByText("Fixture Agent", { exact: true }),
      ).toHaveCount(0);
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= window.innerWidth,
        ),
      ).toBe(true);
      await publication.getByRole("button").first().focus();
      await publication.getByRole("button").first().press("Enter");
      await expect(publication.getByRole("button").first()).toHaveAttribute(
        "aria-expanded",
        "false",
      );
    }
    expect(colors[0]).not.toBe(colors[1]);
  });
}
for (const [story, label] of [
  ["english", "Output"],
  ["japanese", "モデル出力"],
  ["french", "Sortie du modèle"],
] as const) {
  test(`native output has a localized label without a sender: ${story}`, async ({
    page,
  }) => {
    await page.goto(
      `/iframe.html?id=chat-record-presentation--${story}&viewMode=story`,
    );
    await expect(
      page
        .locator('[data-record-origin="assistant"]')
        .first()
        .getByText(label, { exact: true }),
    ).toBeVisible();
    await expect(page.getByText("Fixture Agent", { exact: true })).toHaveCount(
      0,
    );
  });
}
