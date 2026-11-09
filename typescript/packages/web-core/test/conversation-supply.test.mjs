import assert from "node:assert/strict";
import test from "node:test";
import { CaptureStatus, Foreground } from "@chapchoo/rpc-client/conversations";
import { conversationHeader } from "../dist/conversation.js";
import {
  decodeHarnessSupply,
  encodeHarnessSupply,
  decodeSavedSupply,
  sameHarnessSupply,
} from "../dist/conversation-supply.js";
import { harnessSupplyChoice } from "../dist/conversation-types.js";

test("supply choices preserve exact owner paths and never infer an absent choice", () => {
  const existing = { type: "existing", path: "/owner/kept codex" };
  assert.deepEqual(
    decodeHarnessSupply(encodeHarnessSupply(existing)),
    existing,
  );
  assert.deepEqual(decodeSavedSupply({ supply: { managed: {} } }), {
    choice: { type: "managed" },
    requestId: undefined,
  });
  for (const wire of [
    {},
    { managed: {}, existingCodexPath: "/owner/codex" },
    { existingCodexPath: "codex" },
  ])
    assert.throws(() => decodeHarnessSupply(wire));
  for (const path of ["relative", "/owner/\ncode", "/owner/\u0085code"])
    assert.equal(
      harnessSupplyChoice.safeParse({ type: "existing", path }).success,
      false,
    );
  assert.equal(
    sameHarnessSupply(existing, {
      type: "existing",
      path: "/owner/kept codex ",
    }),
    false,
  );
  assert.equal(sameHarnessSupply(existing, { type: "managed" }), false);
});
test("the owner header uses only an exact-source native applied fact", () => {
  const current = {
    conversation: {
      conversationId: "conversation",
      status: CaptureStatus.CAPTURE_STATUS_READY,
      snapshotId: "snapshot",
      nativeSessionId: "native",
    },
    observedNative: {
      target: { snapshotId: "snapshot", nativeSessionId: "native" },
      foreground: Foreground.FOREGROUND_IDLE,
      appliedSupply: { existingCodexPath: "/owner/actual codex" },
    },
  };
  assert.deepEqual(conversationHeader("agent", current).appliedSupply, {
    type: "existing",
    path: "/owner/actual codex",
  });
  assert.equal(
    conversationHeader("agent", { conversation: current.conversation })
      .appliedSupply,
    undefined,
  );
  assert.throws(() =>
    conversationHeader("agent", {
      ...current,
      observedNative: {
        ...current.observedNative,
        target: { snapshotId: "old", nativeSessionId: "native" },
      },
    }),
  );
});
