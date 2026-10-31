import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  toneColors,
  darkToneColors,
  toneMetrics,
  platformFonts,
} from "./src/designTokens.ts";

const document = readFileSync(
  new URL("../../../DESIGN.md", import.meta.url),
  "utf8",
);
const block = /```yaml\n([\s\S]*?)\n```/.exec(document);
assert.ok(block, "DESIGN.md canonical token block is present");
const canonical = JSON.parse(block[1]);

test("owned web token adapter matches the canonical visual document", () => {
  assert.deepEqual(toneColors, canonical.colors);
  assert.deepEqual(darkToneColors, canonical.darkColors);
  assert.deepEqual(toneMetrics.type, canonical.typeScale);
  assert.deepEqual(toneMetrics.layout, canonical.layout);
  for (const [key, value] of Object.entries(toneMetrics.space)) {
    assert.equal(`${value}px`, canonical.spacing[key]);
  }
  for (const [key, value] of Object.entries(toneMetrics.radius)) {
    assert.equal(`${value}px`, canonical.rounded[key]);
  }
  assert.equal(platformFonts.web, canonical.typography["body-web"].fontFamily);
  assert.equal(platformFonts.ios, canonical.typography["body-ios"].fontFamily);
});

function luminance(hex) {
  const channels = hex
    .match(/[a-f\d]{2}/gi)
    .map((value) => parseInt(value, 16) / 255)
    .map((value) =>
      value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4,
    );
  return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
}
function contrast(a, b) {
  const [lo, hi] = [luminance(a), luminance(b)].sort((x, y) => x - y);
  return (hi + 0.05) / (lo + 0.05);
}

test("intended small-text foreground/background pairs meet 4.5:1", () => {
  for (const colors of [toneColors, darkToneColors]) {
    for (const [fg, bg] of [
      ["ink", "canvas"],
      ["secondary", "canvas"],
      ["muted", "surface"],
      ["onAccent", "accent"],
      ["onAccent", "accentHover"],
      ["accentText", "accentSoft"],
      ["ink", "overlay"],
      ["muted", "overlay"],
      ["success", "successSoft"],
      ["warning", "warningSoft"],
      ["danger", "dangerSoft"],
    ]) {
      assert.ok(contrast(colors[fg], colors[bg]) >= 4.5, `${fg} on ${bg}`);
    }
    assert.ok(contrast(colors.focus, colors.surface) >= 3);
    assert.ok(contrast(colors.muted, colors.surfaceWarm) >= 3);
  }
});
