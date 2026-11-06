import { expect, test } from "@playwright/test";

for (const palette of ["rose", "teal"]) {
  test(`Agent ${palette} primary theme reaches portaled dialog controls`, async ({
    page,
  }) => {
    await page.goto(
      `/iframe.html?id=shared-ui-agent-theme--${palette}&viewMode=story`,
    );
    const portal = page.getByRole("button", {
      name: "Portal action",
      exact: true,
    });
    await expect(portal).toBeVisible();
    const swatch = await page.evaluate(
      (key) =>
        getComputedStyle(document.documentElement)
          .getPropertyValue(`--chapchoo-profile-${key}-swatch`)
          .trim(),
      palette,
    );
    const rendered = await portal.evaluate(
      (element) => getComputedStyle(element).backgroundColor,
    );
    const expected = await page.evaluate((color) => {
      const sample = document.createElement("span");
      sample.style.color = color;
      document.body.append(sample);
      const value = getComputedStyle(sample).color;
      sample.remove();
      return value;
    }, swatch);
    expect(rendered).toBe(expected);
    await expect(portal).toHaveCSS(
      "background-color",
      await page
        .getByRole("button", {
          name: "Agent action",
          exact: true,
          includeHidden: true,
        })
        .evaluate((element) => getComputedStyle(element).backgroundColor),
    );
  });
}
