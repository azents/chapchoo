import assert from "node:assert/strict";
import test from "node:test";
import { initialProfile } from "./creationDefaults.ts";

const images = [
  { id: "one", internalName: "One", assetKey: "first" },
  { id: "two", internalName: "Two", assetKey: "second" },
];
void test("a new form draws image and palette independently exactly once", () => {
  const bounds: number[] = [];
  const result = initialProfile(images, (bound) => {
    bounds.push(bound);
    return bounds.length === 1 ? 1 : 4;
  });
  assert.deepEqual(bounds, [2, 6]);
  assert.equal(result.initialImage.id, "two");
  assert.equal(result.initialPalette, "lilac");
});
void test("invalid catalogue or draw has no silent profile default", () => {
  assert.throws(() => initialProfile([], () => 0));
  assert.throws(() => initialProfile(images, () => 99));
});
