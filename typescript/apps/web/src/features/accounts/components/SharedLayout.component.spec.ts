import { expect, test } from "@playwright/test";

for (const width of [1280, 390]) {
  test(`shared form sizes align at ${width}px and preserve enlarged text`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 900 });
    for (const story of [
      "control-sizes",
      "disabled-control-sizes",
      "invalid-control-sizes",
    ]) {
      await page.goto(
        `/iframe.html?id=shared-ui-fields--${story}&viewMode=story`,
      );
      await expect(page.locator("[data-control-pair]")).toHaveCount(35);
      for (const scale of ["100%", "200%"]) {
        await page.evaluate((value) => {
          document.documentElement.style.fontSize = value;
        }, scale);
        await expect(
          page.getByRole("button", { name: "Compact action" }),
        ).toHaveCSS(
          "min-height",
          `${(width === 390 ? 44 : 26) * (scale === "200%" ? 2 : 1)}px`,
        );
        for (const pair of await page.locator("[data-control-pair]").all()) {
          const frame = pair.locator(".mantine-Input-input").first();
          const button = pair.locator(".mantine-Button-root");
          const inputBox = await frame.boundingBox();
          const buttonBox = await button.boundingBox();
          expect(inputBox).not.toBeNull();
          expect(buttonBox).not.toBeNull();
          // Multi-value controls may wrap when text enlarges or space narrows.
          if (
            width === 390 &&
            (await pair.locator(".mantine-MultiSelect-root").count())
          ) {
            expect(inputBox?.height ?? 0).toBeGreaterThanOrEqual(
              (buttonBox?.height ?? 0) - 1,
            );
          } else {
            expect(
              Math.abs((inputBox?.height ?? 0) - (buttonBox?.height ?? 0)),
            ).toBeLessThanOrEqual(1);
          }
          if (width === 390) {
            const editor = pair
              .locator('input:not([type="hidden"]),select')
              .first();
            if (await editor.count()) {
              expect(
                await editor.evaluate((element) =>
                  parseFloat(getComputedStyle(element).fontSize),
                ),
              ).toBeGreaterThanOrEqual(16);
            }
          }
        }
        expect(
          await page
            .getByRole("textbox", { name: "Multiline" })
            .evaluate((element) => element.getBoundingClientRect().height),
        ).toBeGreaterThan(44);
      }
    }
  });
}

test("context divider rows share the section baseline without a filled surface", async ({
  page,
}) => {
  for (const width of [1280, 390, 320]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(
      "/iframe.html?id=accounts-workspace--contexts-and-invitation&viewMode=story",
    );
    const row = page.locator('main [data-variant="row"]').first();
    await expect(row).toBeVisible();
    await expect(row).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
    await expect(row).toHaveCSS("padding-left", "0px");
    const link = await row.getByRole("link").boundingBox();
    const heading = await page
      .getByRole("main")
      .getByRole("heading", { level: 1 })
      .boundingBox();
    expect(link?.x).toBe(heading?.x);
    expect(
      await page.evaluate(
        () =>
          document.documentElement.scrollWidth <=
          document.documentElement.clientWidth,
      ),
    ).toBe(true);
  }
});

test("desktop sidebar and workspace scroll independently, including short windows", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 700 });
  await page.goto(
    "/iframe.html?id=shared-ui-applicationshell--long-content&viewMode=story",
  );
  const sidebar = page.locator("aside");
  const footer = sidebar.locator("img");
  const workspace = page.getByRole("main").locator("..");
  await expect(footer).toBeVisible();
  const initial = await footer.boundingBox();
  await workspace.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  expect(
    await workspace.evaluate((element) => element.scrollTop),
  ).toBeGreaterThan(0);
  expect(await footer.boundingBox()).toEqual(initial);
  expect(await sidebar.evaluate((element) => element.scrollTop)).toBe(0);
  expect(
    await page.evaluate(() => document.documentElement.scrollHeight),
  ).toBeLessThanOrEqual(700);

  await page.setViewportSize({ width: 1280, height: 300 });
  await page.goto(
    "/iframe.html?id=shared-ui-applicationshell--long-navigation&viewMode=story",
  );
  await expect(sidebar.getByRole("link")).toHaveCount(20);
  expect(
    await sidebar.evaluate(
      (element) => element.scrollHeight > element.clientHeight,
    ),
  ).toBe(true);
  await sidebar.evaluate((element) => {
    element.scrollTop = element.scrollHeight;
  });
  expect(
    await sidebar.evaluate((element) => element.scrollTop),
  ).toBeGreaterThan(0);
  const bounds = await sidebar.boundingBox();
  const bottom = await footer.boundingBox();
  expect(bottom?.y).toBeGreaterThanOrEqual(bounds?.y ?? 0);
  expect((bottom?.y ?? 0) + (bottom?.height ?? 0)).toBeLessThanOrEqual(300);
  expect(await workspace.evaluate((element) => element.scrollTop)).toBe(0);

  await page.setViewportSize({ width: 390, height: 700 });
  await page.goto(
    "/iframe.html?id=shared-ui-applicationshell--long-content&viewMode=story",
  );
  await expect(sidebar).toHaveCSS("overflow-y", "visible");
  expect(
    await page.evaluate(() => document.documentElement.scrollHeight),
  ).toBeGreaterThan(700);
});
