import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import test from "node:test";
import postcss from "postcss";
import { mascotArtwork } from "./src/mascot-motion/mascotArtwork.ts";
import { mascots } from "../ui/src/designTokens.ts";

const css = postcss.parse(
  readFileSync(
    new URL(
      "./src/mascot-motion/AgentMascotMotion.module.css",
      import.meta.url,
    ),
    "utf8",
  ),
);

test("all five approved masters have a tight, valid ground-aligned crop", () => {
  assert.equal(Object.keys(mascotArtwork).length, 5);
  for (const [key, artwork] of Object.entries(mascotArtwork)) {
    assert.equal(artwork.file, mascots.find((pet) => pet.key === key)?.file);
    assert.ok(
      existsSync(
        new URL(`../ui/assets/mascots/${artwork.file}`, import.meta.url),
      ),
    );
    const [x, y, width, height] = artwork.viewBox.split(" ").map(Number);
    assert.equal(width, artwork.width);
    assert.equal(height, artwork.height);
    assert.ok(x >= 0 && y >= 0 && x + width <= 1254 && y + height <= 1254);
  }
});

test("the gesture has a 5% peak, a small rebound and a fixed 30-degree ground line", () => {
  const matrices = [];
  css.walkDecls("transform", (decl) => {
    const match = /^matrix\(([^)]+)\)$/.exec(decl.value);
    if (match) {
      matrices.push(match[1].split(",").map(Number));
    }
  });
  assert.equal(matrices.length, 4);
  const [identity, peak, undershoot, rebound] = matrices;
  assert.deepEqual(identity, [1, 0, 0, 1, 0, 0]);
  assert.equal(peak[3], 1.05);
  assert.equal(undershoot[3], 0.9925);
  assert.equal(rebound[3], 1.01);
  for (const [a, b, c, d, tx, ty] of matrices) {
    assert.ok(a > 0 && d > 0);
    assert.equal(b, 0);
    assert.equal(tx, 0);
    assert.equal(ty, 0);
    if (d !== 1) {
      assert.ok(Math.abs(c / -(d - 1) - Math.tan(Math.PI / 6)) < 0.000001);
    }
    for (const x of [-90, 0, 90]) {
      assert.equal(b * x + d * 0 + ty, 0);
    }
  }
});

test("the 1200ms active clip has six seconds of rest and respects reduced motion", () => {
  const source = css.toString();
  assert.match(source, /grounded-stretch 7200ms infinite/);
  assert.match(source, /16\.666667%/);
  assert.ok(Math.abs(7200 * 0.16666667 - 1200) < 0.001);
  const reducedRules = [];
  css.walkAtRules("media", (rule) => {
    if (rule.params === "(prefers-reduced-motion: reduce)") {
      rule.walkDecls("animation", (decl) => reducedRules.push(decl.value));
    }
  });
  assert.deepEqual(reducedRules, ["none"]);
});
