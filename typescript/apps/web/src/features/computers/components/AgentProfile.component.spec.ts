import { expect, test } from "@playwright/test";
import { creationCopy } from "../creationCopy";

const copy = creationCopy("en-US");
test("saved profile is rendered before editing and a failed save retains its controlled draft until cancel", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-profile--interactive&viewMode=story",
  );
  await expect(
    page.getByText("Research companion", { exact: true }),
  ).toBeVisible();
  await expect(page.getByText("@research", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: copy.edit, exact: true }).click();
  const name = page.getByRole("textbox", { name: "Agent name", exact: true });
  await name.fill("Draft companion");
  await expect(
    page.getByRole("textbox", { name: copy.handle, exact: true }),
  ).toHaveAttribute("readonly", "");
  await page.getByRole("button", { name: copy.save, exact: true }).click();
  await expect(page.getByRole("alert")).toHaveText(copy.saveFailed);
  await expect(name).toHaveValue("Draft companion");
  await expect(
    page.getByText("Research companion", { exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(name).toHaveCount(0);
  await page.getByRole("button", { name: copy.edit, exact: true }).click();
  await expect(name).toHaveValue("Research companion");
  await name.fill("Escape draft");
  await page.keyboard.press("Escape");
  await expect(name).toHaveCount(0);
  await page.getByRole("button", { name: copy.edit, exact: true }).click();
  await expect(name).toHaveValue("Research companion");
});
test("saving disables edits while catalogue failure retains the selected image", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-profile--saving&viewMode=story",
  );
  await expect(
    page.getByRole("textbox", { name: "Agent name", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeDisabled();
  await page.goto(
    "/iframe.html?id=computers-agent-profile--catalogue-failure&viewMode=story",
  );
  await expect(
    page.getByRole("combobox", { name: copy.image, exact: true }),
  ).toHaveValue(copy.currentImage);
  await expect(
    page.getByRole("combobox", { name: copy.image, exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("textbox", { name: "Agent name", exact: true }),
  ).toHaveValue("Changed companion");
});
test("Korean saved-profile editing keeps keyboard controls and save reachable on mobile", async ({
  page,
}, info) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto(
    "/iframe.html?id=computers-agent-profile--korean&viewMode=story",
  );
  const ko = creationCopy("ko");
  const name = page.getByRole("textbox", {
    name: "에이전트 이름",
    exact: true,
  });
  await name.focus();
  await expect(name).toBeFocused();
  await expect(
    page.getByRole("button", { name: ko.save, exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
  await page.evaluate(async () => document.fonts.ready);
  await page.screenshot({
    path: info.outputPath("saved-profile-mobile.png"),
    fullPage: true,
  });
});
