import assert from "node:assert/strict";
import test from "node:test";
import { team, teams } from "../dist/validation.js";
import { RpcFailure } from "../dist/errors.js";

const base = {
  id: "opaque-team-id",
  name: "Design",
  ownerUserId: "opaque-user-id",
  callerRole: 1,
};

test("personal absence on the wire becomes explicit browser null, not a synthetic handle", () => {
  const personal = team({ ...base, kind: 1, handle: undefined });
  assert.equal(personal.handle, null);
  assert.equal(JSON.parse(JSON.stringify(personal)).handle, null);
  const result = teams({
    items: [
      { ...base, kind: 1, handle: undefined },
      { ...base, kind: 2, handle: "design" },
    ],
  });
  assert.equal(result.items[0].handle, null);
  assert.equal(result.items[1].handle, "design");
});

test("Team kind and canonical-handle invariants are validated at generated response ingress", () => {
  for (const handle of ["", "personal", null]) {
    assert.throws(() => team({ ...base, kind: 1, handle }), RpcFailure);
  }
  for (const handle of [
    undefined,
    null,
    "",
    "ab",
    "Upper",
    "a_b",
    "abc\n",
    "-abc",
    "abc-",
    "abcdefghijklmnopq",
  ]) {
    assert.throws(() => team({ ...base, kind: 2, handle }), RpcFailure);
  }
  for (const handle of ["abc", "a--b", "abcdefghijklmnop"]) {
    assert.equal(team({ ...base, kind: 2, handle }).handle, handle);
  }
});
