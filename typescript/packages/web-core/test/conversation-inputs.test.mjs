import assert from "node:assert/strict";
import { test } from "node:test";
import { readConversationActivity } from "../dist/conversation.js";
import {
  removeConversationInput,
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
        return {
          inputs: [{ acceptance, totalBytes: BigInt(bytes.length) }],
          latest: acceptance,
        };
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
          delivery: { failed: {} },
        };
      },
      records: async () => ({ records: [] }),
      latestPlan: async () => ({ records: [] }),
      liveRequests: async () => ({ requests: [] }),
      stopResult: async () => ({ receipt: { state: 2 } }),
    },
  };
}
test("a not-started failed input remains completely readable for explicit recovery on reentry", async () => {
  const f = fixture({ started: false });
  const result = await readConversationActivity(f.api, { agentId: "agent" });
  assert.equal(result.header.state, "notStarted");
  assert.equal(result.inputs[0].text, f.text);
  assert.equal(result.inputs[0].observation, "failed");
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
test("exact closure removes a consumed cached input without mistaking pagination for deletion", async () => {
  const f = fixture();
  const previous = await readConversationActivity(f.api, { agentId: "agent" });
  f.api.inputs = async (query) => {
    assert.deepEqual(query.knownIds, [f.acceptance.id]);
    return { inputs: [], closedIds: [f.acceptance.id] };
  };
  const incoming = await readConversationActivity(f.api, {
    agentId: "agent",
    expectedSnapshotId: "snapshot",
    expectedConversationId: "conversation",
    checkInputs: [f.acceptance.id],
  });
  assert.deepEqual(incoming.closedInputs, [f.acceptance.id]);
  assert.deepEqual(mergeConversationRegion(previous, incoming).inputs, []);
  assert.equal(
    mergeConversationRegion(previous, { ...incoming, closedInputs: [] }).inputs
      .length,
    1,
  );
});

test("pickup between listing and status read closes the input rather than fabricating uncertainty", async () => {
  const f = fixture();
  f.api.inputResult = async () => ({});
  f.api.readInput = async () => {
    throw new Error("Consumed text must not be read");
  };
  const result = await readConversationActivity(f.api, { agentId: "agent" });
  assert.deepEqual(result.inputs, []);
  assert.deepEqual(result.closedInputs, [f.acceptance.id]);
  assert.equal(result.latestInput, undefined);
});

test("pickup during byte transfer is confirmed by an exact read; other failures remain failures", async () => {
  for (const consumed of [true, false]) {
    const f = fixture();
    let transferred = false;
    f.api.readInput = async () => {
      transferred = true;
      throw new Error("Transfer interrupted");
    };
    f.api.inputResult = async () =>
      transferred && consumed
        ? {}
        : { acceptance: f.acceptance, ownership: 2, delivery: { failed: {} } };
    if (consumed) {
      const result = await readConversationActivity(f.api, {
        agentId: "agent",
      });
      assert.deepEqual(result.inputs, []);
      assert.deepEqual(result.closedInputs, [f.acceptance.id]);
    } else {
      await assert.rejects(
        readConversationActivity(f.api, { agentId: "agent" }),
        /Transfer interrupted/,
      );
    }
  }
});

test("first native binding retains the original acceptance target without retargeting a mutation", async () => {
  const f = fixture();
  const result = await readConversationActivity(f.api, { agentId: "agent" });
  assert.equal(result.inputs[0].text, f.text);
  assert.deepEqual(f.acceptance.target, { notStarted: {} });
  assert.deepEqual(f.reads[0].input.scope.target, { snapshotId: "snapshot" });
});
test("waiting queue bodies come from the server just like failed bodies", async () => {
  const f = fixture();
  f.api.inputResult = async () => ({
    acceptance: f.acceptance,
    ownership: 1,
  });
  const result = await readConversationActivity(f.api, { agentId: "agent" });
  assert.equal(result.inputs[0].text, f.text);
  assert.equal(result.inputs[0].observation, "accepted");
  assert.ok(f.reads.some((read) => read.type === "text"));
});
test("queue text stays visible while the native source is loading", async () => {
  const f = fixture();
  f.api.current = async () => ({
    conversation: {
      conversationId: "conversation",
      status: 2,
      snapshotId: "snapshot",
    },
  });
  f.api.inputResult = async () => ({
    acceptance: f.acceptance,
    ownership: 1,
  });
  f.api.liveRequests = async () => {
    throw new Error("Loading native history is not queue authority");
  };
  const result = await readConversationActivity(f.api, { agentId: "agent" });
  assert.equal(result.header.state, "loading");
  assert.equal(result.inputs[0].text, f.text);
  assert.deepEqual(result.records, []);
});
test("active updates reuse immutable cached input text while refreshing only actual retained result facts", async () => {
  const f = fixture({ known: true });
  const result = await readConversationActivity(f.api, {
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
  assert.equal(merged.inputs[0].observation, "failed");
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
  const result = await readConversationActivity(f.api, {
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
  const stale = await readConversationActivity(f.api, {
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
    acceptance: {
      ...f.acceptance,
      requestId,
      id: requestId === "latest" ? "latest-operation" : f.acceptance.id,
      acceptedAt: requestId === "latest" ? 40n : f.acceptance.acceptedAt,
    },
    ownership: 1,
    preparation:
      requestId === "latest" ? { possibleForwarding: {} } : { failed: 3 },
  });
  const first = await readConversationActivity(f.api, {
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
  f.api.inputs = async (input) => ({
    ...(await list(input)),
    latest: {
      ...f.acceptance,
      id: "latest-operation",
      requestId: "latest",
      acceptedAt: 40n,
    },
  });
  const older = await readConversationActivity(f.api, {
    agentId: "agent",
    expectedConversationId: "conversation",
    knownInputs: [f.acceptance.id],
    inputBefore: { acceptedAt: 30n, requestId: "cursor" },
  });
  assert.deepEqual(older.latestInput.preparation, { type: "preparing" });
  assert.equal(older.latestInput.requestId, "latest");
  assert.equal(older.inputs[0].preparation.type, "failed");
});
for (const when of ["before-status", "during-body"]) {
  test(`an explicitly reused consumed Request ID cannot replace an old input's body: ${when}`, async () => {
    const f = fixture();
    let reused = when === "before-status";
    const originalRead = f.api.readInput;
    f.api.readInput = async (input) => {
      const result = await originalRead(input);
      reused = true;
      return result;
    };
    f.api.inputResult = async () => ({
      acceptance: reused
        ? { ...f.acceptance, id: "new-operation", acceptedAt: 30n }
        : f.acceptance,
      ownership: 1,
      delivery: { failed: {} },
    });
    const region = await readConversationActivity(f.api, { agentId: "agent" });
    assert.deepEqual(region.inputs, []);
    assert.deepEqual(region.closedInputs, [f.acceptance.id]);
    assert.equal(region.latestInput, undefined);
  });
}

test("acknowledged queue removal preserves other items and checks the exact source", () => {
  const region = {
    header: {
      agentId: "agent",
      conversationId: "conversation",
      state: "notStarted",
    },
    records: [],
    inputs: [
      {
        id: "one",
        requestId: "one",
        conversationId: "conversation",
        acceptedAt: 1n,
        observation: "failed",
        text: "first",
      },
      {
        id: "two",
        requestId: "two",
        conversationId: "conversation",
        acceptedAt: 2n,
        observation: "accepted",
        text: "second",
      },
    ],
    latestInput: { requestId: "two", observation: "accepted" },
    renderedAt: 1,
  };
  assert.equal(removeConversationInput(region, "one", "foreign"), region);
  const removed = removeConversationInput(region, "one", "conversation");
  assert.deepEqual(removed.inputs, [region.inputs[1]]);
  assert.equal(removed.latestInput, region.latestInput);
  const empty = removeConversationInput(removed, "two", "conversation");
  assert.deepEqual(empty.inputs, []);
  assert.equal(empty.latestInput, undefined);
});
