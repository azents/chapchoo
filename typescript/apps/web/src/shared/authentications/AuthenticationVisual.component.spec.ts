import { writeFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";

test("approved authentication surfaces render at native desktop and mobile conditions", async ({
  page,
}, info) => {
  await page.emulateMedia({ colorScheme: "light", reducedMotion: "reduce" });
  const conditions = [];
  for (const viewport of [
    { width: 1440, height: 1100 },
    { width: 390, height: 844 },
  ]) {
    await page.setViewportSize(viewport);
    for (const [name, story] of [
      ["catalogue", "authentication-catalogue--populated"],
      ["environment", "computers-agent-creation--approved-environment"],
      ["authentication", "computers-agent-creation--approved-authentication"],
      ["profile", "computers-agent-creation--approved"],
      ["agent", "computers-agents--approved-detail"],
      ["key", "authentication-connections--key-entry"],
      ["device", "authentication-connections--device-code"],
    ] as const) {
      await page.goto(
        `/iframe.html?id=${story}&viewMode=story&globals=locale:ko-KR;colorScheme:light`,
      );
      await expect(page.getByRole("heading").first()).toBeVisible();
      await page.evaluate(() => document.fonts.ready);
      const metadata = await page.evaluate(() => {
        const element =
          document.querySelector('[role="dialog"] h2') ??
          document.querySelector("#storybook-root main h1");
        if (!element) {
          throw new Error("A rendered heading is required");
        }
        const font = getComputedStyle(element);
        return {
          width: innerWidth,
          height: innerHeight,
          scale: devicePixelRatio,
          fontFamily: font.fontFamily,
          fontSize: font.fontSize,
          fontWeight: font.fontWeight,
          lineHeight: font.lineHeight,
          loadedFonts: document.fonts.status,
          loadedFamilies: [
            ...new Set(
              Array.from(document.fonts)
                .filter((face) => face.status === "loaded")
                .map((face) => face.family),
            ),
          ],
          reflow: document.documentElement.scrollWidth <= innerWidth,
        };
      });
      expect(metadata.scale).toBe(1);
      expect(metadata.loadedFonts).toBe("loaded");
      expect(metadata.fontFamily).toContain("Pretendard");
      expect(
        metadata.loadedFamilies.some((family) => family.includes("Pretendard")),
      ).toBe(true);
      expect(metadata.reflow).toBe(true);
      const file = `${name}-${viewport.width}.png`;
      await page.screenshot({
        path: info.outputPath(file),
        fullPage: true,
        animations: "disabled",
      });
      conditions.push({
        artifact: "component render",
        file,
        story,
        locale: "ko-KR",
        colorScheme: "light",
        reducedMotion: true,
        ...metadata,
      });
    }
  }
  await writeFile(
    info.outputPath("conditions.json"),
    JSON.stringify(conditions, null, 2),
  );
});
