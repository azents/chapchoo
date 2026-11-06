import assert from "node:assert/strict";
import test from "node:test";
import {
  authenticationTargetChanged,
  definitiveAuthenticationRejection,
  withdrawAuthenticationSession,
} from "./failure.ts";
void test("only changed or removed targets discard a rejected management draft", () => {
  for (const reason of [
    "DOMAIN_REASON_CONFLICT",
    "DOMAIN_REASON_INVALID_TRANSITION",
    "DOMAIN_REASON_NOT_FOUND",
  ]) {
    assert.equal(
      authenticationTargetChanged(
        Object.assign(new Error("Rejected"), { data: { reason } }),
      ),
      true,
    );
  }
  for (const reason of [
    "DOMAIN_REASON_INVALID_INPUT",
    "DOMAIN_REASON_INVALID_SESSION",
    "DOMAIN_REASON_CAPACITY_EXHAUSTED",
    null,
  ]) {
    assert.equal(
      authenticationTargetChanged(
        Object.assign(new Error("Rejected"), { data: { reason } }),
      ),
      false,
    );
  }
});
void test("only explicit non-acceptance unlocks another authentication attempt", () => {
  for (const reason of [
    "DOMAIN_REASON_INVALID_INPUT",
    "DOMAIN_REASON_PERMISSION_DENIED",
    "DOMAIN_REASON_NOT_FOUND",
    "DOMAIN_REASON_CONFLICT",
    "DOMAIN_REASON_INVALID_TRANSITION",
    "DOMAIN_REASON_CAPACITY_EXHAUSTED",
  ]) {
    assert.equal(
      definitiveAuthenticationRejection(
        Object.assign(new Error("Rejected"), { data: { reason } }),
      ),
      true,
    );
  }
  for (const value of [
    new Error("Transport unavailable"),
    null,
    { data: { reason: "DOMAIN_REASON_INVALID_INPUT" } },
    Object.assign(new Error("Uncertain"), { data: { reason: null } }),
  ]) {
    assert.equal(definitiveAuthenticationRejection(value), false);
  }
});
void test("imperative secret mutation failure withdraws identity only for invalid sessions", () => {
  let withdrawn = 0;
  const clear = (): void => {
    withdrawn += 1;
  };
  assert.equal(
    withdrawAuthenticationSession(
      Object.assign(new Error("Rejected"), {
        data: { reason: "DOMAIN_REASON_INVALID_SESSION" },
      }),
      clear,
    ),
    true,
  );
  assert.equal(withdrawn, 1);
  assert.equal(
    withdrawAuthenticationSession(
      Object.assign(new Error("Denied"), {
        data: { reason: "DOMAIN_REASON_PERMISSION_DENIED" },
      }),
      clear,
    ),
    false,
  );
  assert.equal(
    withdrawAuthenticationSession(new Error("Uncertain"), clear),
    false,
  );
  assert.equal(withdrawn, 1);
});
