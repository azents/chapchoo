import assert from "node:assert/strict";
import test from "node:test";
import { inventoryInput } from "./inventory.ts";

void test("inventory query inputs are JSON-safe and preserve the full backend int64 range", () => {
  const input = { offset: "9223372036854775807", limit: 25 };
  assert.doesNotThrow(() => JSON.stringify(input));
  assert.deepEqual(inventoryInput.parse(input), {
    offset: 9223372036854775807n,
    limit: 25n,
  });
  assert.deepEqual(inventoryInput.parse({}), {});
});

void test("inventory input rejects noncanonical offsets, overflow and invalid page sizes", () => {
  for (const offset of [
    "-1",
    "01",
    "1.5",
    "1e2",
    "9223372036854775808",
    "9".repeat(100),
  ]) {
    assert.equal(inventoryInput.safeParse({ offset }).success, false);
  }
  for (const limit of [0, -1, 1.5, 1001, "25"]) {
    assert.equal(inventoryInput.safeParse({ limit }).success, false);
  }
  assert.equal(inventoryInput.safeParse({ offset: 0n }).success, false);
  assert.equal(inventoryInput.safeParse({ unknown: true }).success, false);
});
