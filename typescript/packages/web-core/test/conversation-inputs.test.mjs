import assert from "node:assert/strict";
import { test } from "node:test";
import { readConversationRegion } from "../dist/conversation.js";
import {
  markConversationInputDeleted,
  mergeConversationRegion,
} from "../dist/conversation-state.js";
function fixture({ started = true, known = false } = {}) {
  const text = `  Original 🙂 ${"가".repeat(22000)}\n `;
  const bytes = Buffer.from(text);
  const reads = [];
  const source = started
    ? { snapshotId: "snapshot", nativeSessionId: "native" }
    : {};
  const acceptance = {
    id: "input-operation",
    conversationId: "conversation",
    requestId: "request",
    acceptedAt: 20n,
    target: { notStarted: {} },
  };
  return {
    text,
    reads,
    acceptance,
    api: {
      current: async () => ({
        conversation: {
          conversationId: "conversation",
          status: started ? 3 : 1,
          ...source,
        },
      }),
      inputs: async (input) => {
        reads.push({ type: "list", input });
        return { inputs: [{ acceptance, totalBytes: BigInt(bytes.length) }] };
      },
      readInput: async (input) => {
        assert.equal(known, false);
        reads.push({ type: "text", input });
        return {
          content: bytes.subarray(input.offset, input.offset + input.length),
          offset: input.offset,
          totalBytes: BigInt(bytes.length),
        };
      },
      inputResult: async (input) => {
        reads.push({ type: "status", input });
        return {
          acceptance,
          ownership: 2,
          delivery: { possibleForwarding: {} },
        };
      },
      records: async () => ({ records: [] }),
      latestPlan: async () => ({ records: [] }),
      liveRequests: async () => ({ requests: [] }),
      stopResult: async () => ({ receipt: { state: 2 } }),
    },
  };
}
test("a not-started accepted input remains completely readable with truthful delivery on reentry", async () => {
  const f = fixture({ started: false });
  const result = await readConversationRegion(f.api, { agentId: "agent" });
  assert.equal(result.header.state, "notStarted");
  assert.equal(result.inputs[0].text, f.text);
  assert.equal(result.inputs[0].observation, "uncertain");
  assert.deepEqual(f.reads[0].input.scope.target, { notStarted: {} });
  assert.ok(
    f.reads
      .filter((read) => read.type === "text")
      .every((read) => read.input.length === 32768),
  );
  assert.equal(
    f.reads.filter((read) => read.type === "text").at(-1).input.offset,
    Buffer.byteLength(f.text),
  );
});
test("first native binding retains the original acceptance target without retargeting a mutation", async () => {
  const f = fixture();
  const result = await readConversationRegion(f.api, { agentId: "agent" });
  assert.equal(result.inputs[0].text, f.text);
  assert.deepEqual(f.acceptance.target, { notStarted: {} });
  assert.deepEqual(f.reads[0].input.scope.target, { snapshotId: "snapshot" });
});
test("active updates reuse immutable cached input text while refreshing only actual retained result facts", async () => {
  const f = fixture({ known: true });
  const result = await readConversationRegion(f.api, {
    agentId: "agent",
    expectedSnapshotId: "snapshot",
    expectedConversationId: "conversation",
    knownInputs: [f.acceptance.id],
  });
  assert.equal(result.inputs[0].text, undefined);
  assert.deepEqual(
    f.reads.map((read) => read.type),
    ["list", "status"],
  );
  const merged = mergeConversationRegion(
    {
      ...result,
      inputs: [{ ...result.inputs[0], text: "unchanged original" }],
    },
    result,
  );
  assert.equal(merged.inputs[0].text, "unchanged original");
  assert.equal(merged.inputs[0].observation, "uncertain");
});
test("deleted input bodies are not reread and a tombstone removes cached text after reentry", async () => {
  const f = fixture({ started: false, known: true });
  f.api.inputs = async () => ({
    inputs: [{ acceptance: f.acceptance, totalBytes: 0n, deleted: true }],
  });
  const result = await readConversationRegion(f.api, { agentId: "agent" });
  assert.equal(result.inputs[0].deleted, true);
  assert.equal(result.inputs[0].text, undefined);
  assert.equal(result.latestInput, undefined);
  const merged = mergeConversationRegion(
    {
      ...result,
      inputs: [
        { ...result.inputs[0], deleted: false, text: "must not return" },
      ],
    },
    result,
  );
  assert.equal(merged.inputs[0].deleted, true);
  assert.equal(merged.inputs[0].text, undefined);
  assert.equal(
    markConversationInputDeleted(merged, merged.inputs[0].id, "another"),
    merged,
  );
  const restored = {
    ...result,
    inputs: [{ ...result.inputs[0], text: "cached body", deleted: false }],
  };
  const ack = markConversationInputDeleted(
    restored,
    result.inputs[0].id,
    "conversation",
  );
  assert.equal(ack.inputs[0].deleted, true);
  assert.equal(ack.inputs[0].text, undefined);
  assert.equal(
    mergeConversationRegion(ack, restored).inputs[0].text,
    undefined,
  );
  assert.equal(
    mergeConversationRegion(ack, { ...restored, inputs: [] }).inputs[0].text,
    undefined,
  );
  const paged = await readConversationRegion(f.api, {
    agentId: "agent",
    expectedConversationId: "conversation",
    inputBefore: { acceptedAt: 1n, requestId: "old" },
  });
  assert.equal(paged.latestInput, undefined);
});
test("source replacement discards cached retained inputs instead of reviving initial-target receipts", () => {
  const f = fixture();
  const before = {
    header: {
      agentId: "agent",
      state: "ready",
      source: {
        agentId: "agent",
        conversationId: "conversation",
        snapshotId: "old",
      },
    },
    records: [],
    inputs: [{ id: "old", text: "old retained input" }],
  };
  const after = {
    header: {
      ...before.header,
      source: { ...before.header.source, snapshotId: "new" },
    },
    records: [],
    inputs: [],
    refill: true,
  };
  assert.deepEqual(mergeConversationRegion(before, after).inputs, []);
  assert.equal(f.acceptance.target.notStarted !== undefined, true);
});
test("Stop result observation is read-only and limited to its exact source and conversation", async () => {
  const f = fixture();
  let reads = 0;
  f.api.stopResult = async (input) => {
    reads++;
    assert.equal(input.requestId, "stop-request");
    return { receipt: { state: 2 } };
  };
  const result = await readConversationRegion(f.api, {
    agentId: "agent",
    stop: {
      conversationId: "conversation",
      requestId: "stop-request",
      snapshotId: "snapshot",
    },
  });
  assert.deepEqual(result.stop, {
    requestId: "stop-request",
    observation: "published",
  });
  const stale = await readConversationRegion(f.api, {
    agentId: "agent",
    stop: {
      conversationId: "conversation",
      requestId: "stop-request",
      snapshotId: "stale",
    },
  });
  assert.equal(stale.stop, undefined);
  assert.equal(reads, 1);
});
test("latest startup diagnosis stays current while browsing older failed inputs and survives reentry", async () => {
  const f = fixture({ started: false, known: true });
  f.api.inputResult = async ({ requestId }) => ({
    acceptance: { ...f.acceptance, requestId },
    ownership: 1,
    preparation:
      requestId === "latest" ? { possibleForwarding: {} } : { failed: 3 },
  });
  const first = await readConversationRegion(f.api, {
    agentId: "agent",
    expectedConversationId: "conversation",
    knownInputs: [f.acceptance.id],
  });
  assert.deepEqual(first.latestInput.preparation, {
    type: "failed",
    reason: "nativeRejected",
  });
  assert.deepEqual(first.inputs[0].preparation, first.latestInput.preparation);
  const list = f.api.inputs;
  f.api.inputs = async (input) =>
    input.limit === 1
      ? {
          inputs: [
            {
              acceptance: { ...f.acceptance, requestId: "latest" },
              totalBytes: 0n,
            },
          ],
        }
      : list(input);
  const older = await readConversationRegion(f.api, {
    agentId: "agent",
    expectedConversationId: "conversation",
    knownInputs: [f.acceptance.id],
    inputBefore: { acceptedAt: 30n, requestId: "cursor" },
  });
  assert.deepEqual(older.latestInput.preparation, { type: "preparing" });
  assert.equal(older.latestInput.requestId, "latest");
  assert.equal(older.inputs[0].preparation.type, "failed");
});
