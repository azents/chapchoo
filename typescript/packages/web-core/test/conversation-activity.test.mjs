import assert from "node:assert/strict";
import { test } from "node:test";
import { conversationActivity } from "../dist/conversation-activity.js";
import { inputPreparation } from "../dist/conversation.js";

function region(change = {}) {
  return {
    header: {
      agentId: "agent",
      conversationId: "conversation",
      state: "notStarted",
      foreground: "unavailable",
      savedAuthenticationId: "auth",
    },
    records: [],
    inputs: [],
    renderedAt: 1,
    ...change,
  };
}
function failure(reason = "nativeRejected") {
  return {
    requestId: "failed",
    observation: "failed",
    preparation: { type: "failed", reason },
  };
}
test("missing native state alone is not execution failure; selection, startup and confirmed setup failures differ", () => {
  assert.equal(conversationActivity(region()), "notStarted");
  assert.equal(
    conversationActivity(
      region({
        header: { ...region().header, startup: { type: "preparing" } },
      }),
    ),
    "starting",
  );
  assert.equal(
    conversationActivity(
      region({
        header: {
          ...region().header,
          startup: { type: "failed", reason: "nativeRejected" },
        },
      }),
    ),
    "startFailed",
  );
  assert.equal(
    conversationActivity(
      region({
        header: {
          ...region().header,
          startup: { type: "failed", reason: "sourceUnavailable" },
        },
      }),
    ),
    "sourceUnavailable",
  );
  const unauthenticated = region({
    header: {
      ...region().header,
      savedAuthenticationId: undefined,
      startup: { type: "preparing" },
    },
  });
  assert.equal(conversationActivity(unauthenticated), "authenticationRequired");
  const missing = region();
  delete missing.header.savedAuthenticationId;
  assert.equal(conversationActivity(missing), "authenticationRequired");
  assert.equal(
    conversationActivity(
      region({ latestInput: { requestId: "new", observation: "preparing" } }),
    ),
    "starting",
  );
  assert.equal(
    conversationActivity(region({ latestInput: failure() })),
    "startFailed",
  );
  assert.equal(
    conversationActivity(region({ latestInput: failure("notAdmitted") })),
    "startFailed",
  );
  assert.equal(
    conversationActivity(region({ latestInput: failure("launchFailed") })),
    "startFailed",
  );
  assert.equal(
    conversationActivity(region({ latestInput: failure("sourceUnavailable") })),
    "sourceUnavailable",
  );
  assert.equal(
    conversationActivity(region({ latestInput: failure("retirementUnknown") })),
    "retirementUnknown",
  );
  assert.equal(
    conversationActivity(
      region({ latestInput: { requestId: "new", observation: "failed" } }),
    ),
    "deliveryFailed",
  );
  assert.equal(
    conversationActivity(
      region({ latestInput: { requestId: "new", observation: "uncertain" } }),
    ),
    "deliveryUnconfirmed",
  );
});
test("failed connection startup remains visible during loading and after first acceptance", () => {
  const value = region();
  value.header.state = "loading";
  value.header.startup = { type: "failed", reason: "sourceUnavailable" };
  assert.equal(conversationActivity(value), "sourceUnavailable");
  value.latestInput = { requestId: "first", observation: "accepted" };
  assert.equal(conversationActivity(value), "sourceUnavailable");
});
test("fresh native observations win over historical failures and missing saved selection", () => {
  for (const foreground of ["idle", "working", "requiresAction"]) {
    const value = region({ latestInput: failure() });
    value.header = {
      ...value.header,
      foreground,
      savedAuthenticationId: undefined,
      state: "ready",
    };
    assert.equal(conversationActivity(value), foreground);
  }
});
test("new local acceptance replaces older failed diagnosis, but uncertain input never authorizes replay", () => {
  const value = region({ latestInput: failure() });
  for (const observation of ["sending", "accepted", "preparing"]) {
    assert.equal(
      conversationActivity(value, [
        { requestId: "new", text: "new", observation },
      ]),
      "starting",
    );
  }
  assert.equal(
    conversationActivity(value, [
      { requestId: "new", text: "new", observation: "uncertain" },
    ]),
    "deliveryUnconfirmed",
  );
  assert.equal(
    conversationActivity(value, [
      {
        requestId: "new",
        snapshotId: "other",
        text: "new",
        observation: "sending",
      },
    ]),
    "startFailed",
  );
  assert.equal(
    conversationActivity(value, [
      {
        requestId: "new",
        conversationId: "another",
        text: "new",
        observation: "accepted",
      },
    ]),
    "startFailed",
  );
  assert.equal(
    conversationActivity(value, [
      { requestId: "failed", text: "new", observation: "accepted" },
    ]),
    "startFailed",
  );
  value.inputs = [{ requestId: "new" }];
  assert.equal(
    conversationActivity(value, [
      { requestId: "new", text: "new", observation: "sending" },
    ]),
    "startFailed",
  );
});
test("typed setup decoding retains reported facts without guessing token or network causes", () => {
  for (const [failed, reason] of [
    [1, "notAdmitted"],
    [2, "launchFailed"],
    [3, "nativeRejected"],
    [4, "sourceUnavailable"],
    [5, "retirementUnknown"],
  ]) {
    assert.deepEqual(
      inputPreparation({ acceptance: {}, preparation: { failed } }),
      { type: "failed", reason },
    );
  }
  assert.equal(inputPreparation({ preparation: { failed: 3 } }), undefined);
  assert.equal(inputPreparation({ acceptance: {} }), undefined);
  assert.deepEqual(
    inputPreparation({
      acceptance: {},
      preparation: { possibleForwarding: {} },
    }),
    { type: "preparing" },
  );
  assert.deepEqual(
    inputPreparation({ acceptance: {}, preparation: { sessionReady: {} } }),
    { type: "ready" },
  );
  assert.throws(() =>
    inputPreparation({ acceptance: {}, preparation: { failed: 99 } }),
  );
});
