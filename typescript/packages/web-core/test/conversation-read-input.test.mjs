import assert from "node:assert/strict";
import test from "node:test";
import {
  conversationReadAnchor,
  conversationReadInputAnchor,
  conversationReadInteger,
} from "../dist/conversation-types.js";

test("conversation read request hints are JSON-safe exact int64 decimal strings", () => {
  const hint = { ordinal: "9223372036854775807", index: "0" };
  const wire = JSON.parse(JSON.stringify(hint));
  assert.deepEqual(conversationReadAnchor.parse(wire), {
    ordinal: 9223372036854775807n,
    index: 0n,
  });
  assert.deepEqual(
    conversationReadInputAnchor.parse({
      acceptedAt: "9007199254740993",
      requestId: "original",
    }),
    { acceptedAt: 9007199254740993n, requestId: "original" },
  );
  assert.equal(
    conversationReadInteger.parse("-9223372036854775808"),
    -9223372036854775808n,
  );
  for (const input of [
    "01",
    "-0",
    "+1",
    "1.0",
    " 1",
    "1e3",
    "9223372036854775808",
    "-9223372036854775809",
    1,
    1n,
  ]) {
    assert.equal(conversationReadInteger.safeParse(input).success, false);
  }
  assert.equal(
    conversationReadAnchor.safeParse({ ordinal: "0", index: "0" }).success,
    false,
  );
  assert.equal(
    conversationReadAnchor.safeParse({ ordinal: "1", index: "-1" }).success,
    false,
  );
});
