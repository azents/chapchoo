import { expect, test } from "@playwright/test";

test("large catalogue characters animate automatically, stop offscreen and respect reduced motion", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(
    "/iframe.html?id=shared-character-artwork--all-characters&viewMode=story",
  );
  const clips = page.locator("[data-playing]");
  await expect(clips).toHaveCount(6);
  await expect(page.locator('[data-playing="true"]')).toHaveCount(0);
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await expect(page.locator('[data-playing="true"]')).toHaveCount(6);
  await clips.evaluateAll((nodes) =>
    nodes.forEach((node) =>
      node.getAnimations().forEach((animation) => animation.pause()),
    ),
  );
  await clips
    .first()
    .locator("..")
    .evaluate((node) => {
      node.style.marginTop = "200vh";
    });
  await expect(clips.first()).toHaveAttribute("data-playing", "false");
  await clips
    .first()
    .locator("..")
    .evaluate((node) => {
      node.style.marginTop = "0";
    });
  await expect(clips.first()).toHaveAttribute("data-playing", "true");
  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect(page.locator('[data-playing="true"]')).toHaveCount(0);
});

test("decorative motion adds no replay button and leaves the Agent link usable", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=shared-character-artwork--inside-navigation&viewMode=story",
  );
  const link = page.getByRole("link", { name: "Open Agent", exact: true });
  await expect(link.locator("[data-playing]")).toHaveCount(1);
  await expect(link.locator("button, [tabindex]")).toHaveCount(0);
  await page.route("**/chosen-agent", (route) =>
    route.fulfill({ body: "Chosen Agent", contentType: "text/plain" }),
  );
  await link.press("Enter");
  await expect(page).toHaveURL(/\/chosen-agent$/);
});
