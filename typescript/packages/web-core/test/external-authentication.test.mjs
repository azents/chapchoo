import assert from "node:assert/strict";
import test from "node:test";
import {
  authentication,
  authenticationAccepted,
  authenticationRecordResult,
  authenticationSelectionResult,
  authenticationDeletion,
  authenticationHistory,
  authenticationList,
  authenticationSelection,
  authenticationScope,
  deviceProgress,
  wireAuthenticationScope,
} from "../dist/external-authentication.js";

const instant = { seconds: 1n, nanos: 0 };
void test("mutation boundaries require their operation-specific accepted identity", () => {
  const recordResult = { record: { authenticationId: "credential" } };
  const selectionResult = {
    selection: { agentId: "agent", authenticationId: "credential" },
  };
  assert.equal(authenticationRecordResult(recordResult).type, "record");
  assert.equal(
    authenticationSelectionResult(selectionResult).type,
    "selection",
  );
  assert.throws(() => authenticationRecordResult(selectionResult));
  assert.throws(() => authenticationSelectionResult(recordResult));
});
const record = {
  id: "credential",
  scope: { personal: {} },
  kind: 2,
  name: "Development",
  currentRequestId: "register",
  createdAt: instant,
  updatedAt: instant,
};
void test("catalogue projection contains only safe identity and preserves exact scope", () => {
  const value = authentication({ ...record, key: "not-a-browser-field" });
  assert.deepEqual(value, {
    id: "credential",
    scope: { type: "personal" },
    kind: "openaiApiKey",
    name: "Development",
    accountId: null,
    email: null,
    planType: null,
    expiresAt: null,
    currentRequestId: "register",
    createdAt: 1000,
    updatedAt: 1000,
  });
  assert.deepEqual(authenticationScope({ teamId: "team" }), {
    type: "team",
    teamId: "team",
  });
  assert.deepEqual(wireAuthenticationScope({ type: "team", teamId: "team" }), {
    teamId: "team",
  });
  assert.throws(() => authenticationScope({ personal: {}, teamId: "team" }));
  assert.throws(() => authenticationScope(undefined));
  assert.throws(() => authentication({ ...record, kind: 0 }));
  assert.throws(() =>
    authentication({ ...record, email: "unexpected@example.com" }),
  );
  assert.equal(authentication({ ...record, kind: 1 }).accountId, null);
  assert.equal(
    authentication({
      ...record,
      kind: 1,
      accountId: "account",
      email: "person@example.com",
    }).accountId,
    "account",
  );
  assert.equal(
    authenticationList({ items: [record], nextAfter: "credential" }).nextAfter,
    "credential",
  );
  assert.throws(() => authenticationList({ items: Array(21).fill(record) }));
});
void test("saved selection preserves request fencing without application state", () => {
  assert.deepEqual(
    authenticationSelection({
      authenticationId: "credential",
      name: "Development",
      kind: 2,
      requestId: "accepted",
      selectedAt: instant,
    }),
    {
      authenticationId: "credential",
      name: "Development",
      kind: "openaiApiKey",
      requestId: "accepted",
      selectedAt: 1000,
    },
  );
});
void test("uncertain recovery distinguishes no observation from an accepted operation", () => {
  assert.deepEqual(authenticationHistory({ operation: 0 }), {
    type: "notObserved",
  });
  const accepted = { record: { authenticationId: "credential" } };
  assert.deepEqual(
    authenticationHistory({ operation: 1, accepted, acceptedAt: instant }),
    {
      type: "accepted",
      operation: "registerKey",
      result: { type: "record", authenticationId: "credential" },
      acceptedAt: 1000,
    },
  );
  assert.throws(() =>
    authenticationHistory({ operation: 4, accepted, acceptedAt: instant }),
  );
  assert.throws(() => authenticationHistory({ operation: 1 }));
  assert.throws(() =>
    authenticationAccepted({
      ...accepted,
      deleted: { authenticationId: "credential" },
    }),
  );
  assert.deepEqual(authenticationDeletion({ inUse: {} }), { type: "inUse" });
  assert.throws(() => authenticationDeletion({ accepted }));
});
void test("device boundary admits only one state and the approved fixed destination", () => {
  const base = { attemptId: "attempt", expiresAt: instant };
  const pending = {
    userCode: "ABCD-EFGH",
    verificationUri: "https://auth.openai.com/codex/device",
    nextCheckAt: instant,
  };
  assert.equal(deviceProgress({ ...base, pending }).state.type, "pending");
  assert.deepEqual(
    deviceProgress({ ...base, registered: { authenticationId: "credential" } })
      .state,
    { type: "registered", authenticationId: "credential" },
  );
  assert.equal(
    deviceProgress({ ...base, uncertain: {}, providerFailure: 4 })
      .providerFailure,
    "uncertain",
  );
  assert.throws(() =>
    deviceProgress({
      ...base,
      pending: {
        ...pending,
        verificationUri: "https://untrusted.invalid/device",
      },
    }),
  );
  assert.throws(() => deviceProgress({ ...base, pending, checking: {} }));
  assert.throws(() => deviceProgress(base));
  assert.throws(() =>
    deviceProgress({ ...base, expired: {}, providerFailure: 0 }),
  );
  assert.throws(() =>
    deviceProgress({
      ...base,
      expiresAt: { seconds: 253402300800n, nanos: 0 },
      expired: {},
    }),
  );
});
