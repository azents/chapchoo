import { expect, test } from "@playwright/test";
import { creationCopy } from "../creationCopy";

const copy = creationCopy("en-US");
test("profile choices update the draft preview, failed save retains it, and cancel/Escape restore saved values and focus", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-profile--interactive&viewMode=story",
  );
  await expect(
    page.getByText("Research companion", { exact: true }),
  ).toBeVisible();
  const edit = page.getByRole("button", { name: copy.edit, exact: true });
  await edit.click();
  const name = page.getByRole("textbox", { name: copy.name, exact: true });
  const images = page.getByRole("group", { name: copy.image, exact: true });
  const colors = page.getByRole("group", { name: copy.palette, exact: true });
  await expect(images.getByRole("button")).toHaveCount(6);
  const retriever = images.getByRole("button", {
    name: "Retriever",
    exact: true,
  });
  await retriever.focus();
  await page.keyboard.press("Space");
  await expect(retriever).toHaveAttribute("aria-pressed", "true");
  await colors
    .getByRole("button", { name: copy.palettes.butter, exact: true })
    .click();
  await expect(
    colors.getByRole("button", { name: copy.palettes.butter, exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await colors
    .getByRole("button", { name: copy.randomPalette, exact: true })
    .click();
  await expect(
    colors.getByRole("button", { name: copy.palettes.sage, exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await name.fill("Draft companion");
  const preview = page.getByRole("region", { name: copy.preview, exact: true });
  await expect(preview).toContainText("Draft companion");
  await expect(preview.locator("img")).toHaveAttribute(
    "src",
    "/design-assets/01-retriever-216.webp",
  );
  const handle = page.getByRole("group", { name: copy.handle, exact: true });
  await expect(handle).toContainText("@research");
  await expect(handle).toContainText(copy.handlePreserved);
  await page.getByRole("button", { name: copy.save, exact: true }).click();
  await expect(page.getByRole("alert")).toHaveText(copy.saveFailed);
  await expect(name).toHaveValue("Draft companion");
  await expect(
    page.getByRole("heading", { name: "Research companion", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(name).toHaveCount(0);
  await expect(edit).toBeFocused();
  await edit.click();
  await expect(name).toHaveValue("Research companion");
  await expect(
    images.getByRole("button", { name: "Corgi", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await name.fill("Escape draft");
  await page.keyboard.press("Escape");
  await expect(name).toHaveCount(0);
  await expect(edit).toBeFocused();
  await edit.click();
  await expect(name).toHaveValue("Research companion");
  await name.fill("Close draft");
  await page.getByRole("button", { name: copy.close, exact: true }).click();
  await expect(name).toHaveCount(0);
  await expect(edit).toBeFocused();
  await edit.click();
  await expect(name).toHaveValue("Research companion");
});
test("saving disables edits while catalogue failure retains the selected image", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-profile--saving&viewMode=story",
  );
  await expect(
    page.getByRole("textbox", { name: copy.name, exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Cancel", exact: true }),
  ).toBeDisabled();
  const images = page.getByRole("group", { name: copy.image, exact: true });
  await expect(images.getByRole("button")).toHaveCount(6);
  for (const button of await images.getByRole("button").all()) {
    await expect(button).toBeDisabled();
  }
  await expect(
    page.getByRole("button", { name: copy.randomPalette, exact: true }),
  ).toBeDisabled();
  await page.goto(
    "/iframe.html?id=computers-agent-profile--catalogue-failure&viewMode=story",
  );
  const retained = images.getByRole("button", {
    name: copy.currentImage,
    exact: true,
  });
  await expect(retained).toHaveAttribute("aria-pressed", "true");
  await expect(retained).toBeDisabled();
  await expect(
    page.getByRole("textbox", { name: copy.name, exact: true }),
  ).toHaveValue("Changed companion");
});
test("Korean editing keeps six compact image choices, keyboard input and save reachable on mobile", async ({
  page,
}, info) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=computers-agent-profile--korean&viewMode=story",
  );
  const ko = creationCopy("ko");
  const name = page.getByRole("textbox", { name: ko.name, exact: true });
  await name.focus();
  await expect(name).toBeFocused();
  await expect(
    page.getByRole("button", { name: ko.save, exact: true }),
  ).toBeVisible();
  const choices = page
    .getByRole("group", { name: ko.image, exact: true })
    .getByRole("button");
  await expect(choices).toHaveCount(6);
  const rows = await choices.evaluateAll((elements) =>
    elements.map((element) => Math.round(element.getBoundingClientRect().top)),
  );
  expect(new Set(rows).size).toBe(1);
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
