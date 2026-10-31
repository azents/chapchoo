import { expect, test } from "@playwright/test";

test("the shared header keeps its appearance action rightmost and remains sticky", async ({
  page,
}) => {
  for (const width of [1280, 390, 320]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(
      "/iframe.html?id=shared-ui-app-header--signed-in&viewMode=story",
    );
    const header = page.getByRole("banner");
    const mode = header.getByRole("button", {
      name: "Appearance",
      exact: true,
    });
    await expect(mode).toBeVisible();
    const before = await header.boundingBox();
    const target = await mode.boundingBox();
    if (!before || !target) {
      throw new Error("Visible header geometry is required");
    }
    expect(target.x + target.width).toBeLessThanOrEqual(
      before.x + before.width,
    );
    for (const action of await header.getByRole("button").all()) {
      const box = await action.boundingBox();
      if (box) {
        expect(box.x).toBeLessThanOrEqual(target.x);
      }
    }
    await page.evaluate(() =>
      window.scrollTo(0, document.documentElement.scrollHeight),
    );
    expect((await header.boundingBox())?.y).toBe(before.y);
  }
});

test("long identity remains inside a growing header at 200% text size", async ({
  page,
}) => {
  for (const width of [769, 800]) {
    await page.setViewportSize({ width, height: 950 });
    for (const id of ["shared-ui-app-header--long-identity"]) {
      await page.goto(
        `/iframe.html?id=${id}&viewMode=story&globals=colorScheme:dark`,
      );
      const header = page.getByRole("banner");
      const mode = header.getByRole("button", {
        name: "Appearance",
        exact: true,
      });
      await expect(mode).toBeVisible();
      await page.evaluate(() => {
        document.documentElement.style.fontSize = "200%";
      });
      const box = await header.boundingBox();
      const identity = await header
        .getByText(
          "long-account-name-with-an-unbroken-identifier-that-must-remain-readable@example.com",
          { exact: true },
        )
        .boundingBox();
      expect(box).not.toBeNull();
      expect(identity).not.toBeNull();
      if (box && identity) {
        expect(identity.y).toBeGreaterThanOrEqual(box.y);
        expect(identity.y + identity.height).toBeLessThanOrEqual(
          box.y + box.height,
        );
      }
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth),
      ).toBeLessThanOrEqual(width);
    }
  }
});
