import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { reconcilePresence } from "./chatPresence.ts";
import type { PresenceEntry } from "./chatPresence.ts";

function item(key: string): {
  key: string;
  node: ReturnType<typeof createElement>;
} {
  return { key, node: createElement("div", { key }, key) };
}
const initial: PresenceEntry[] = ["a", "b"].map((key) => ({
  ...item(key),
  order: key === "a" ? 1n : 2n,
  present: true,
  initial: true,
}));

void test("polling updates content without replaying entry", () => {
  const next = [item("a"), item("b")];
  const result = reconcilePresence(initial, next);
  assert.deepEqual(
    result.map(({ key, present, initial }) => ({ key, present, initial })),
    [
      { key: "a", present: true, initial: true },
      { key: "b", present: true, initial: true },
    ],
  );
  assert.equal(result[0]?.node, next[0]?.node);
});

void test("new tail enters while prepended history is immediately visible", () => {
  const result = reconcilePresence(initial, [
    { ...item("older"), order: 0n },
    item("a"),
    item("b"),
    { ...item("new"), order: 3n },
  ]);
  assert.equal(result[0]?.initial, true);
  assert.equal(result[3]?.initial, false);
});

void test("exit retains its position, latest content and can reverse", () => {
  const result = reconcilePresence(initial, [item("b")]);
  assert.deepEqual(
    result.map((entry) => [entry.key, entry.present]),
    [
      ["a", false],
      ["b", true],
    ],
  );
  assert.equal(result[0]?.node, initial[0]?.node);
  const reversed = reconcilePresence(result, [item("a"), item("b")]);
  assert.equal(reversed.length, 2);
  assert.equal(reversed[0]?.present, true);
});

void test("pending-to-published input identity does not create a second bubble", () => {
  const pending = [{ ...item("input:1"), present: true, initial: false }];
  const result = reconcilePresence(pending, [item("input:1")]);
  assert.equal(result.length, 1);
  assert.equal(result[0]?.present, true);
});
