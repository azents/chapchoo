import assert from "node:assert/strict";
import { test } from "node:test";
import { hasMessageArrival } from "./recordArrival.ts";

void test("new-record notification excludes prepends, updates, removal and source replacement", () => {
  const previous = {
    source: "conversation/snapshot",
    tail: 4n,
  };
  assert.equal(hasMessageArrival(previous, { ...previous }), false);
  assert.equal(
    hasMessageArrival(previous, {
      ...previous,
      tail: 3n,
    }),
    false,
  );
  assert.equal(hasMessageArrival(previous, { ...previous, tail: null }), false);
  assert.equal(
    hasMessageArrival(previous, {
      ...previous,
      source: "replacement",
      tail: 100n,
    }),
    false,
  );
  assert.equal(
    hasMessageArrival(previous, {
      ...previous,
      tail: 5n,
    }),
    true,
  );
  assert.equal(
    hasMessageArrival(previous, {
      ...previous,
      tail: 6n,
    }),
    true,
  );
  assert.equal(
    hasMessageArrival({ source: previous.source, tail: null }, previous),
    true,
  );
});
