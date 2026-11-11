import assert from "node:assert/strict";
import { test } from "node:test";
import { hasRecordArrival } from "./recordArrival.ts";

void test("new-record notification excludes prepends, updates, removal and source replacement", () => {
  const previous = {
    source: "conversation/snapshot",
    tail: { ordinal: 4n, index: 2n },
  };
  assert.equal(hasRecordArrival(previous, { ...previous }), false);
  assert.equal(
    hasRecordArrival(previous, {
      ...previous,
      tail: { ordinal: 4n, index: 1n },
    }),
    false,
  );
  assert.equal(hasRecordArrival(previous, { ...previous, tail: null }), false);
  assert.equal(
    hasRecordArrival(previous, {
      ...previous,
      source: "replacement",
      tail: { ordinal: 100n, index: 0n },
    }),
    false,
  );
  assert.equal(
    hasRecordArrival(previous, {
      ...previous,
      tail: { ordinal: 4n, index: 3n },
    }),
    true,
  );
  assert.equal(
    hasRecordArrival(previous, {
      ...previous,
      tail: { ordinal: 5n, index: 0n },
    }),
    true,
  );
  assert.equal(
    hasRecordArrival({ source: previous.source, tail: null }, previous),
    true,
  );
});
