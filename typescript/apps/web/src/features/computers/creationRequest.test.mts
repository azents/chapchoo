import assert from "node:assert/strict";
import test from "node:test";
import { CreationRequest } from "./creationRequest.ts";
import type { CreationDraft } from "./creationRequest.ts";

const draft: CreationDraft = {
  teamId: "team",
  providerId: "provider",
  authenticationId: "authentication",
  name: "Companion",
  handle: "companion",
  profileImageId: "selected-image",
  paletteKey: "sage",
};
void test("uncertain creation retains every immutable field and its original request", () => {
  const lock = new CreationRequest();
  const first = lock.capture(draft, () => "original-request");
  lock.reject();
  assert.ok(lock.locked);
  for (const reason of [
    null,
    "DOMAIN_REASON_INVALID_SESSION",
    "DOMAIN_REASON_CONFLICT",
  ]) {
    lock.reject(reason);
    assert.ok(lock.locked);
    const recovered = lock.capture(
      {
        teamId: "other-team",
        providerId: "other-provider",
        authenticationId: "other-authentication",
        name: "Changed",
        handle: "changed",
        profileImageId: "other-image",
        paletteKey: "rose",
      },
      () => {
        throw new Error("Uncertain retry must not issue a request");
      },
    );
    assert.equal(recovered, first);
    assert.deepEqual(recovered, { ...draft, requestId: "original-request" });
  }
});
void test("definitive pre-acceptance rejection permits a new complete attempt", () => {
  for (const reason of [
    "DOMAIN_REASON_INVALID_INPUT",
    "DOMAIN_REASON_INVALID_TRANSITION",
    "DOMAIN_REASON_PERMISSION_DENIED",
    "DOMAIN_REASON_NOT_FOUND",
    "DOMAIN_REASON_HANDLE_CONFLICT",
  ]) {
    const lock = new CreationRequest();
    lock.capture(draft, () => "first");
    lock.reject(reason);
    assert.equal(lock.locked, false);
    assert.equal(
      lock.capture({ ...draft, handle: "another" }, () => "next").requestId,
      "next",
    );
  }
});
void test("recovering a removed outcome unlocks without allocating a replacement", () => {
  const lock = new CreationRequest();
  lock.capture(draft, () => "first");
  lock.removed();
  assert.equal(lock.locked, false);
});
void test("deleted selected authentication before final admission unlocks explicit reselection", () => {
  const lock = new CreationRequest();
  const first = lock.capture(draft, () => "deleted-reference");
  lock.reject("DOMAIN_REASON_NOT_FOUND");
  assert.equal(lock.locked, false);
  const next = lock.capture(
    { ...draft, authenticationId: "new-reference" },
    () => "new-explicit-request",
  );
  assert.notEqual(next, first);
  assert.equal(next.authenticationId, "new-reference");
  assert.equal(next.requestId, "new-explicit-request");
});
