import assert from "node:assert/strict";
import { test } from "node:test";
import { MessageRole } from "@chapchoo/rpc-client/conversations";
import { readConversationMessages } from "../dist/conversation-messages.js";
import {
  acknowledgeConversationInputs,
  mergeConversationMessages,
} from "../dist/conversation-message-state.js";
import { conversationMessageWindow } from "../dist/conversation-message-types.js";

function metadata(id, order, text = id) {
  return {
    id,
    receiptOrder: order,
    role: MessageRole.MESSAGE_ROLE_USER,
    receivedAt: order + 100n,
    totalBytes: BigInt(Buffer.byteLength(text)),
    sourceInputId: `input-${id}`,
    sourceInputRequestId: "reused-nonce",
  };
}
function fixture(items, through, text = new Map()) {
  const reads = [];
  const current = {
    conversation: { conversationId: "conversation", status: 1 },
  };
  const api = {
    current: async () => current,
    messages: async (input) => {
      reads.push(input);
      return { messages: items, through };
    },
    readMessage: async (input) => {
      const body = Buffer.from(text.get(input.messageId) ?? input.messageId);
      return {
        offset: input.offset,
        totalBytes: BigInt(body.length),
        content: body.subarray(input.offset, input.offset + input.length),
      };
    },
    records: () => {
      throw new Error("Message chat must not read native records");
    },
    fields: () => {
      throw new Error("Message chat must not read native fields");
    },
  };
  return { api, reads, current };
}
const latest = { agentId: "agent", window: { type: "latest" } };

test("message reads preserve exact UTF-8 across byte chunks and source replacement", async () => {
  const body = `${"x".repeat(32767)}🙂\n끝  `;
  const { api, current } = fixture(
    [metadata("message", 1n, body)],
    1n,
    new Map([["message", body]]),
  );
  const read = api.readMessage;
  const offsets = [];
  api.readMessage = async (input) => {
    assert.deepEqual(input.scope, {
      agentId: "agent",
      conversationId: "conversation",
    });
    assert.equal(input.messageId, "message");
    assert.equal(input.length, 32768);
    offsets.push(input.offset);
    // Native readiness/identity has no authority over this product message.
    current.conversation.status = 2;
    current.conversation.snapshotId = "replacement";
    return read(input);
  };
  const result = await readConversationMessages(api, latest);
  assert.equal(result.messages[0].text, body);
  assert.equal(result.messages[0].sourceInputRequestId, "reused-nonce");
  assert.deepEqual(offsets, [0, 32768, Buffer.byteLength(body)]);
});

test("absent conversation is a read-only empty region with no creation or native read", async () => {
  const { api, reads } = fixture([], 0n);
  api.current = async () => ({ conversation: undefined });
  const result = await readConversationMessages(api, latest);
  assert.equal(result.conversationId, undefined);
  assert.deepEqual(result.messages, []);
  assert.equal(result.through, 0n);
  assert.equal(reads.length, 0);
});

test("after catch-up freezes its finite head and preserves int64 order precision", async () => {
  const start = 9007199254740993n;
  const items = Array.from({ length: 25 }, (_, i) =>
    metadata(`m-${i}`, start + BigInt(i + 1)),
  );
  const { api, reads } = fixture(items, start + 40n);
  const result = await readConversationMessages(api, {
    agentId: "agent",
    expectedConversationId: "conversation",
    window: { type: "after", order: start, through: start + 40n },
  });
  assert.deepEqual(reads[0].after, { order: start, through: start + 40n });
  assert.equal(result.messages[0].order, start + 1n);
  assert.deepEqual(result.next, { order: start + 25n, through: start + 40n });
  assert.equal(result.older, undefined);
  api.messages = async () => ({ messages: [], through: start + 41n });
  await assert.rejects(
    readConversationMessages(api, {
      agentId: "agent",
      window: { type: "after", order: start, through: start + 40n },
    }),
    /boundary changed/u,
  );
});

test("a selected conversation change resets the requested window, not the message target", async () => {
  const { api, reads } = fixture([metadata("new", 1n)], 1n);
  const result = await readConversationMessages(api, {
    agentId: "agent",
    expectedConversationId: "old",
    window: { type: "before", order: 100n },
  });
  assert.equal(result.window.type, "latest");
  assert.equal(reads[0].before, undefined);
  assert.equal(reads[0].scope.conversationId, "conversation");
  assert.equal(result.messages[0].id, "new");
});

test("missing or changed bodies and changed product scope are explicit failures", async () => {
  const { api } = fixture([metadata("message", 1n)], 1n);
  api.readMessage = async () => {
    throw new Error("Authorized body unavailable");
  };
  await assert.rejects(
    readConversationMessages(api, latest),
    /body unavailable/u,
  );
  api.readMessage = async () => ({
    offset: 0,
    totalBytes: 100n,
    content: Buffer.from("message"),
  });
  await assert.rejects(
    readConversationMessages(api, latest),
    /continuation changed/u,
  );
  const selected = fixture([metadata("message", 1n)], 1n);
  let calls = 0;
  selected.api.current = async () => ({
    conversation: {
      conversationId: ++calls === 1 ? "conversation" : "another",
    },
  });
  await assert.rejects(
    readConversationMessages(selected.api, latest),
    /selection changed/u,
  );
});

test("message metadata rejects unknown roles, duplicate identities and nonascending orders", async () => {
  for (const items of [
    [{ ...metadata("a", 1n), role: MessageRole.MESSAGE_ROLE_UNSPECIFIED }],
    [metadata("a", 1n), metadata("a", 2n)],
    [metadata("a", 2n), metadata("b", 1n)],
  ]) {
    const { api } = fixture(items, 2n);
    await assert.rejects(readConversationMessages(api, latest));
  }
});

function region(messages, extra = {}) {
  return {
    agentId: "agent",
    conversationId: "conversation",
    messages,
    window: { type: "latest" },
    through: 2n,
    renderedAt: 1,
    ...extra,
  };
}
function message(id, order) {
  return {
    id,
    order,
    role: "user",
    receivedAt: order,
    text: "same text",
    sourceInputRequestId: "reused-nonce",
  };
}
test("merging uses immutable receipt identity, never equal text or reused request nonce", () => {
  const first = region([message("a", 1n)]);
  const next = region([message("b", 2n)], {
    window: { type: "after", order: 1n },
  });
  const result = mergeConversationMessages(first, next);
  assert.deepEqual(
    result.messages.map((item) => item.id),
    ["a", "b"],
  );
  assert.throws(
    () =>
      mergeConversationMessages(
        result,
        region([{ ...message("a", 1n), text: "changed" }]),
      ),
    /Immutable/u,
  );
  assert.throws(
    () => mergeConversationMessages(result, region([message("c", 2n)])),
    /order conflicts/u,
  );
  assert.deepEqual(
    mergeConversationMessages(result, region([], { conversationId: "another" }))
      .messages,
    [],
  );
});

test("older paging cannot advance an unread head or discard a finite catch-up", () => {
  const current = region([message("latest", 10n)], {
    through: 30n,
    older: 10n,
    next: { order: 20n, through: 30n },
  });
  const older = region([message("old", 1n)], {
    window: { type: "before", order: 10n },
    through: 50n,
    older: undefined,
  });
  const merged = mergeConversationMessages(current, older);
  assert.equal(merged.through, 30n);
  assert.deepEqual(merged.next, { order: 20n, through: 30n });
  assert.equal(merged.older, undefined);
  const delta = region([message("new", 31n)], {
    window: { type: "after", order: 30n },
    through: 31n,
  });
  assert.equal(mergeConversationMessages(merged, delta).older, undefined);
});

test("a lost acceptance response recovers only the current outstanding receipt after input pickup", () => {
  const local = [
    {
      requestId: "outstanding",
      text: "same text",
      observation: "uncertain",
      conversationId: "conversation",
    },
  ];
  const receipt = {
    ...message("receipt", 1n),
    sourceInputId: "consumed-input",
    sourceInputRequestId: "outstanding",
  };
  const recovered = acknowledgeConversationInputs(
    local,
    "agent",
    region([receipt]),
  );
  assert.deepEqual(recovered.acknowledged, local);
  assert.deepEqual(recovered.inputs, [
    { ...local[0], id: "consumed-input", observation: "accepted" },
  ]);
  // A receipt confirms acceptance, not native insertion or durable nonce recovery.
  assert.equal(recovered.inputs[0].snapshotId, undefined);
  assert.deepEqual(
    acknowledgeConversationInputs(recovered.inputs, "agent", region([receipt]))
      .acknowledged,
    [],
  );
  assert.equal(
    acknowledgeConversationInputs([], "agent", region([receipt])).inputs.length,
    0,
  );
  for (const unrelated of [
    region([receipt], { agentId: "another-agent" }),
    region([receipt], { conversationId: "another-conversation" }),
    region([{ ...receipt, sourceInputRequestId: "another-current-operation" }]),
    region([{ ...receipt, role: "agent" }]),
    region([
      receipt,
      {
        ...receipt,
        id: "other-receipt",
        sourceInputId: "reused-input",
        order: 2n,
      },
    ]),
  ])
    assert.equal(
      acknowledgeConversationInputs(local, "agent", unrelated).inputs,
      local,
    );
  const known = [{ ...local[0], id: "consumed-input" }];
  const reused = region([
    receipt,
    {
      ...receipt,
      id: "other-receipt",
      sourceInputId: "reused-input",
      order: 2n,
    },
  ]);
  assert.equal(
    acknowledgeConversationInputs(known, "agent", reused).inputs[0].id,
    "consumed-input",
  );
  assert.equal(
    mergeConversationMessages(region([receipt]), {
      ...reused,
      window: { type: "after", order: 1n },
    }).messages.length,
    2,
  );
});

test("latest reset retains a reachable older cursor rather than merging across a gap", () => {
  const previous = region([message("old", 10n)], { older: 10n });
  const latestPage = region([message("new", 50n)], {
    older: 50n,
    through: 50n,
  });
  const merged = mergeConversationMessages(previous, latestPage);
  assert.deepEqual(
    merged.messages.map((item) => item.id),
    ["new"],
  );
  assert.equal(merged.older, 50n);
});

test("creation after an explicitly absent selection starts from the latest window", async () => {
  const { api, reads } = fixture([metadata("new", 50n)], 50n);
  const result = await readConversationMessages(api, {
    agentId: "agent",
    expectedConversationId: null,
    window: { type: "after", order: 0n },
  });
  assert.equal(result.window.type, "latest");
  assert.equal(reads[0].after, undefined);
});

test("browser windows validate canonical int64 strings without floating-point coercion", () => {
  assert.equal(
    conversationMessageWindow.parse({
      type: "after",
      order: "9007199254740993",
      through: "9223372036854775807",
    }).order,
    9007199254740993n,
  );
  for (const order of ["01", "1.5", "9223372036854775808", "-1"]) {
    assert.equal(
      conversationMessageWindow.safeParse({ type: "before", order }).success,
      false,
    );
  }
});
