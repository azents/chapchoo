import { expect, test } from "@playwright/test";

test("custom appearance callbacks restore menu-owned focus but preserve an outside target", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=common-ui-appearance--interactive&viewMode=story",
  );
  const trigger = page.getByRole("button", { name: "Appearance", exact: true });
  const choices: ReadonlyArray<readonly [string, string]> = [
    ["Light", "light"],
    ["Dark", "dark"],
    ["System", "system"],
  ];
  for (const [label, value] of choices) {
    await trigger.click();
    await expect(
      page.getByRole("menuitem", { name: label, exact: true }),
    ).toBeVisible();
    await page.getByRole("menuitem", { name: label, exact: true }).click();
    await expect(trigger).toHaveAttribute("data-mode", value);
    await expect(trigger).toBeFocused();
  }
  await trigger.click();
  await page
    .getByRole("textbox", { name: "Draft" })
    .click({ position: { x: 8, y: 8 } });
  await expect(page.getByRole("textbox", { name: "Draft" })).toBeFocused();
});
