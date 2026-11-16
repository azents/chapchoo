import { expect, test } from "@playwright/test";

for (const story of ["observation-gap", "initial-observation"]) {
  test(`observation gaps do not become generic unknown status captions/${story}`, async ({
    page,
  }) => {
    await page.goto(
      `/iframe.html?id=computers-computer-controls--${story}&viewMode=story`,
    );
    await expect(page.getByText("Unknown", { exact: true })).toHaveCount(0);
    await expect(
      page.getByText("Current power is not verified.", { exact: true }),
    ).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Stop", exact: true }),
    ).toBeDisabled();
    await expect(
      page.getByLabel("Refreshing Computer information", { exact: true }),
    ).toBeVisible();
    await page
      .getByRole("button", { name: "Computer details", exact: true })
      .click();
    const dialog = page.getByRole("dialog", {
      name: "Computer details",
      exact: true,
    });
    await expect(
      dialog.getByText("Power has not been confirmed", { exact: true }),
    ).toHaveCount(0);
    await expect(dialog.getByText("Unknown", { exact: true })).toHaveCount(0);
    await expect(
      dialog.getByRole("button", { name: "Refresh", exact: true }),
    ).toBeEnabled();
    if (story === "observation-gap") {
      await expect(
        dialog.getByText("Last-known power: Running", { exact: true }),
      ).toBeVisible();
    }
  });
}

test("Chat activity sheet does not regress to the old unknown power copy", async ({
  page,
}) => {
  await page.goto("/iframe.html?id=chat-conversation--unknown&viewMode=story");
  await page.getByRole("button", { name: /코기.*작업 상태/ }).click();
  const sheet = page.getByRole("dialog", { name: "작업 상태", exact: true });
  await expect(
    sheet.getByText(/확인되지 않음|현재 전원 상태를 확인하지 못했습니다/),
  ).toHaveCount(0);
  await expect(sheet.getByText("연결됨", { exact: true })).toBeVisible();
});
