import assert from "node:assert/strict";
import { test } from "node:test";
import {
  CaptureStatus,
  Foreground,
  RecordKind,
  RecordField,
  TypedAnswer,
  PreparationFreshness,
} from "@chapchoo/rpc-client/conversations";
import {
  readConversationBytes,
  readConversationRegion,
  readConversationRecord,
  encodeAnswer,
  decodeAnswer,
  submitConversationInput,
  submitConversationAnswer,
  conversationHeader,
} from "../dist/conversation.js";
import { mergeConversationRegion } from "../dist/conversation-state.js";
import {
  initialQuestionDraft,
  reviewQuestionAnswer,
} from "../dist/conversation-form.js";
const source = {
  agentId: "agent",
  conversationId: "conversation",
  snapshotId: "snapshot",
};
const native = { snapshotId: "snapshot", nativeSessionId: "native" };
const header = {
  agentId: "agent",
  conversationId: "conversation",
  source,
  native,
  state: "ready",
  foreground: "working",
  workId: "work",
};
function current(snapshotId = "snapshot") {
  return {
    conversation: {
      conversationId: "conversation",
      snapshotId,
      nativeSessionId: "native",
      status: CaptureStatus.CAPTURE_STATUS_READY,
    },
    observedNative: {
      target: { snapshotId, nativeSessionId: "native" },
      foreground: Foreground.FOREGROUND_RUNNING,
      workId: "work",
    },
  };
}
function metadata(
  kind = RecordKind.RECORD_KIND_ASSISTANT,
  nativeId = "message",
  ordinal = 1n,
) {
  return {
    kind,
    nativeId,
    anchor: { ordinal, index: 0n },
    updatedOrdinal: ordinal,
  };
}
function apiFor(content, records = [metadata()]) {
  const body = Buffer.from(
    content.map((item) => JSON.stringify(item)).join("\n") + "\n",
  );
  const calls = [];
  return {
    calls,
    current: async () => current(),
    records: async () => ({ records }),
    inputs: async () => ({ inputs: [] }),
    latestPlan: async () => ({ records: [] }),
    liveRequests: async () => ({ requests: [] }),
    fields: async () => {
      calls.push("fields");
      return {
        fields: [
          {
            field: RecordField.RECORD_FIELD_CONTENT,
            updatedOrdinal: 1n,
            totalBytes: BigInt(body.length),
          },
        ],
      };
    },
    readField: async ({ offset, length }) => {
      calls.push("read");
      return {
        offset,
        totalBytes: BigInt(body.length),
        content: body.subarray(offset, offset + length),
      };
    },
  };
}
test("source-qualified preparation and applied authentication remain independent of fresh saved selection", () => {
  const value = current();
  value.savedAuthenticationId = "saved-one";
  value.observedNative.authenticationId = "saved-one";
  value.observedNative.preparation = {
    freshness:
      PreparationFreshness.PREPARATION_FRESHNESS_TRANSIENT_LOOKUP_FAILURE,
    node: "v24.0.0",
    adapter: "2.6.0",
    codex: "0.165.0",
  };
  const observed = conversationHeader("agent", value);
  assert.deepEqual(observed.preparation, {
    freshness: "cached",
    node: "v24.0.0",
    adapter: "2.6.0",
    codex: "0.165.0",
  });
  value.savedAuthenticationId = "saved-two";
  const fresh = conversationHeader("agent", value);
  assert.equal(fresh.savedAuthenticationId, "saved-two");
  assert.equal(fresh.appliedAuthenticationId, "saved-one");
  assert.deepEqual(fresh.preparation, observed.preparation);
  const historical = conversationHeader("agent", current());
  assert.equal(historical.preparation, undefined);
  assert.equal(historical.appliedAuthenticationId, undefined);
  value.observedNative.preparation.freshness = 99;
  assert.throws(() => conversationHeader("agent", value));
});
test("bounded bytes preserve UTF-8 split within a code point and recheck the final empty suffix", async () => {
  const value = Buffer.from("다음 🙂 ".repeat(6000));
  const offsets = [];
  const result = await readConversationBytes(async (offset) => {
    offsets.push(offset);
    return {
      offset,
      totalBytes: BigInt(value.length),
      content: value.subarray(offset, offset + 32768),
    };
  });
  assert.deepEqual(result, value);
  assert.equal(offsets.at(-1), value.length);
});
test("mixed lengths, unexpected offsets and incomplete bodies never publish partial text", async () => {
  for (const response of [
    { offset: 1, totalBytes: 1n, content: Buffer.from("x") },
    { offset: 0, totalBytes: 2n, content: Buffer.alloc(0) },
    { offset: 0, totalBytes: 1n, content: Buffer.from("xx") },
  ])
    await assert.rejects(readConversationBytes(async () => response));
  await assert.rejects(
    readConversationBytes(async (offset) => ({
      offset,
      totalBytes: offset === 0 ? 2n : 3n,
      content: Buffer.from("x"),
    })),
  );
});
test("twenty appending large bodies finish their observed extent and advance only the read known hint", async () => {
  for (let iteration = 0; iteration < 20; iteration++) {
    const observedOrdinal = BigInt(2 + iteration * 2);
    const text = `body-${iteration}: 한글\0🙂 `.repeat(6000);
    const observed = Buffer.from(JSON.stringify({ type: "text", text }) + "\n");
    assert.ok(observed.length > 2 * 32768);
    let currentBody = observed;
    let appended = false;
    let fieldReads = 0;
    const offsets = [];
    const api = apiFor([]);
    api.fields = async () => ({
      fields: [
        {
          field: RecordField.RECORD_FIELD_CONTENT,
          updatedOrdinal: observedOrdinal + (appended ? 1n : 0n),
          totalBytes: BigInt(currentBody.length),
        },
        ...(appended
          ? [
              {
                field: RecordField.RECORD_FIELD_TITLE,
                updatedOrdinal: observedOrdinal + 1n,
                totalBytes: 0n,
              },
            ]
          : []),
      ],
    });
    api.readField = async ({ field, updatedOrdinal, offset, length }) => {
      fieldReads++;
      offsets.push(offset);
      assert.equal(field, RecordField.RECORD_FIELD_CONTENT);
      assert.equal(updatedOrdinal, observedOrdinal);
      assert.ok(length <= 32768);
      // Append strictly between the first chunk and its continuation. The port
      // retains the initial extent over the same current byte projection.
      if (offset === 0) {
        currentBody = Buffer.concat([
          currentBody,
          Buffer.from(JSON.stringify({ type: "text", text: "later" }) + "\n"),
        ]);
        appended = true;
      }
      return {
        offset,
        totalBytes: BigInt(observed.length),
        content: currentBody.subarray(
          offset,
          Math.min(offset + length, observed.length),
        ),
      };
    };
    const region = await readConversationRegion(api, { agentId: "agent" });
    assert.ok(appended);
    assert.deepEqual(region.records[0].content, [{ type: "text", text }]);
    assert.equal(region.records[0].updatedOrdinal, observedOrdinal);
    assert.equal(region.records[0].title, undefined);
    assert.equal(offsets.at(-1), observed.length);
    const readsBefore = fieldReads;
    api.records = async () => ({
      records: [metadata(undefined, undefined, observedOrdinal)],
    });
    const unchanged = await readConversationRegion(api, {
      agentId: "agent",
      expectedSnapshotId: "snapshot",
      expectedConversationId: "conversation",
      known: [
        {
          key: region.records[0].key,
          updatedOrdinal: region.records[0].updatedOrdinal,
        },
      ],
    });
    assert.deepEqual(unchanged.records, []);
    assert.equal(fieldReads, readsBefore);
  }
});
test("replacement of an unread extent and final source withdrawal reject complete publication", async () => {
  const text = "large🙂".repeat(12000);
  const replaced = apiFor([{ type: "text", text }]);
  const read = replaced.readField;
  replaced.readField = async (request) => {
    if (request.offset > 0)
      throw new Error("Observed field extent was replaced");
    return read(request);
  };
  await assert.rejects(
    readConversationRecord(replaced, header, metadata()),
    /extent was replaced/,
  );
  const withdrawn = apiFor([{ type: "text", text: "complete" }]);
  const fields = withdrawn.fields;
  let metadataReads = 0;
  withdrawn.fields = async () => {
    if (++metadataReads > 1) throw new Error("Source is no longer readable");
    return fields();
  };
  await assert.rejects(
    readConversationRecord(withdrawn, header, metadata()),
    /no longer readable/,
  );
});
test("initial region reads actual complete content; distinct utterance identity is never text deduplication", async () => {
  const api = apiFor(
    [{ type: "text", text: "same" }],
    [metadata(RecordKind.RECORD_KIND_ASSISTANT, "second", 2n), metadata()],
  );
  // Both fixtures share the same field update, which remains <= their record metadata.
  const region = await readConversationRegion(api, { agentId: "agent" });
  assert.deepEqual(
    region.records.map((record) => record.nativeId),
    ["message", "second"],
  );
  assert.equal(region.records[0].content[0].text, "same");
  assert.equal(region.header.foreground, "working");
});
test("unchanged bodies coalesce but current native/read authority is still reread", async () => {
  const api = apiFor([{ type: "text", text: "body" }]);
  const region = await readConversationRegion(api, {
    agentId: "agent",
    expectedSnapshotId: "snapshot",
    expectedConversationId: "conversation",
    known: [{ key: "assistant:message", updatedOrdinal: 1n }],
  });
  assert.deepEqual(region.records, []);
  assert.deepEqual(api.calls, []);
});
test("new source clears old data before any replacement body is requested", async () => {
  const api = apiFor([{ type: "text", text: "new" }]);
  api.current = async () => current("replacement");
  const region = await readConversationRegion(api, {
    agentId: "agent",
    expectedSnapshotId: "snapshot",
    expectedConversationId: "conversation",
  });
  assert.equal(region.refill, true);
  assert.equal(region.header.source.snapshotId, "replacement");
  assert.deepEqual(region.records, []);
  assert.deepEqual(api.calls, []);
});
test("a source replacement during a complete read rejects that region", async () => {
  const api = apiFor([{ type: "text", text: "body" }]);
  let count = 0;
  api.current = async () => current(count++ ? "replacement" : "snapshot");
  await assert.rejects(
    readConversationRegion(api, { agentId: "agent" }),
    /source changed/,
  );
});
test("late older pages add missing records but never overwrite a newer body", () => {
  const record = {
    key: "assistant:message",
    nativeId: "message",
    kind: "assistant",
    anchor: { ordinal: 1n, index: 0n },
    updatedOrdinal: 9n,
    content: [{ type: "text", text: "new" }],
    locations: [],
  };
  const previous = { header, records: [record], renderedAt: 1 };
  const incoming = {
    header,
    records: [
      {
        ...record,
        updatedOrdinal: 2n,
        content: [{ type: "text", text: "old" }],
      },
    ],
    renderedAt: 2,
  };
  assert.equal(
    mergeConversationRegion(previous, incoming).records[0].content[0].text,
    "new",
  );
  assert.deepEqual(
    mergeConversationRegion(previous, {
      ...incoming,
      header: { ...header, source: { ...source, snapshotId: "new" } },
      records: [],
    }).records,
    [],
  );
});
test("byte-bound input upload preserves leading/trailing text and retains one exact effect identity", async () => {
  const pieces = [];
  const text = `  ${"가".repeat(40000)}\n  `;
  const result = await submitConversationInput(
    {
      submitInput: async (piece) => {
        pieces.push(piece);
        const end = piece.offset + piece.content.length;
        return end === piece.totalBytes
          ? {
              accepted: {
                id: "operation",
                requestId: piece.requestId,
                conversationId: "conversation",
              },
            }
          : { uploading: { nextOffset: end } };
      },
    },
    { agentId: "agent", requestId: "request", text },
  );
  assert.equal(result.id, "operation");
  assert.equal(
    Buffer.concat(pieces.map((piece) => piece.content)).toString(),
    text,
  );
  assert.ok(
    pieces.length > 1 &&
      pieces.every(
        (piece) =>
          piece.content.length <= 32768 &&
          piece.requestId === "request" &&
          piece.target.notStarted,
      ),
  );
});
test("a lost upload response never redispatches an effect", async () => {
  let calls = 0;
  await assert.rejects(
    submitConversationInput(
      {
        submitInput: async () => {
          calls++;
          throw new Error("lost");
        },
      },
      { agentId: "agent", requestId: "same-request", text: "original" },
    ),
  );
  assert.equal(calls, 1);
});
test("typed retained answers preserve absent/empty object, false, empty text, arrays and full i64", () => {
  for (const answer of [
    { type: "formAccept" },
    { type: "formAccept", content: {} },
    {
      type: "formAccept",
      content: {
        a: { type: "boolean", value: false },
        b: { type: "text", value: "" },
        c: { type: "integer", value: 9223372036854775807n },
        d: { type: "strings", value: ["x", "y"] },
      },
    },
    { type: "permission", optionId: "exact:scope" },
    { type: "formDecline" },
    { type: "formCancel" },
    { type: "permissionCancel" },
  ])
    assert.deepEqual(
      decodeAnswer(
        TypedAnswer.decode(TypedAnswer.encode(encodeAnswer(answer)).finish()),
      ),
      answer,
    );
});
test("answer uploads preserve the admitted native/responder/source identities", async () => {
  const pieces = [];
  const answer = {
    type: "formAccept",
    content: { text: { type: "text", value: "🙂".repeat(30000) } },
  };
  await submitConversationAnswer(
    {
      submitAnswer: async (piece) => {
        pieces.push(piece);
        const end = piece.offset + piece.content.length;
        return end === piece.totalBytes
          ? { accepted: { id: "answer", delivery: 1 } }
          : { uploading: { nextOffset: end } };
      },
    },
    {
      source,
      target: native,
      responderToken: "responder",
      requestId: "request",
      answer,
    },
  );
  assert.ok(
    pieces.every(
      (piece) =>
        piece.target === native &&
        piece.responderToken === "responder" &&
        piece.requestId === "request",
    ),
  );
  assert.deepEqual(
    decodeAnswer(
      TypedAnswer.decode(Buffer.concat(pieces.map((piece) => piece.content))),
    ),
    answer,
  );
});
test("forms do not preselect permission and preserve explicit false/empty versus omission", () => {
  const question = {
    type: "form",
    title: "Question",
    fields: [
      { name: "enabled", title: "Enabled", type: "boolean", required: true },
      { name: "text", title: "Text", type: "text", required: true },
      { name: "optional", title: "Optional", type: "number", required: false },
    ],
  };
  const draft = initialQuestionDraft(question);
  assert.deepEqual(reviewQuestionAnswer(question, draft).errors, {
    enabled: "required",
    text: "required",
  });
  draft.fields = {
    enabled: { type: "boolean", value: false },
    text: { type: "text", value: "" },
  };
  assert.deepEqual(reviewQuestionAnswer(question, draft).answer.content, {
    enabled: { type: "boolean", value: false },
    text: { type: "text", value: "" },
  });
  assert.equal(
    initialQuestionDraft({
      type: "permission",
      title: "Allow?",
      options: [{ id: "allow", title: "Allow", scope: "once" }],
    }).optionId,
    undefined,
  );
});
test("form review checks code-point lengths, pattern, formats, numeric and multiple-choice bounds", () => {
  const fields = [
    {
      name: "text",
      title: "Text",
      required: true,
      type: "text",
      minLength: 2,
      maxLength: 3,
      pattern: "^가+.$",
    },
    {
      name: "integer",
      title: "Integer",
      required: true,
      type: "integer",
      minimum: 9223372036854775806n,
      maximum: 9223372036854775807n,
    },
    {
      name: "options",
      title: "Options",
      required: true,
      type: "strings",
      minItems: 1,
      maxItems: 1,
      choices: [{ title: "Title", value: "exact" }],
    },
  ];
  const question = { type: "form", title: "Review", fields };
  const draft = {
    ...initialQuestionDraft(question),
    fields: {
      text: { type: "text", value: "가🙂" },
      integer: { type: "text", value: "9223372036854775807" },
      options: { type: "strings", value: ["exact"] },
    },
  };
  assert.deepEqual(reviewQuestionAnswer(question, draft).errors, {});
  draft.fields.integer.value = "9223372036854775808";
  draft.fields.options.value = ["unoffered"];
  assert.deepEqual(reviewQuestionAnswer(question, draft).errors, {
    integer: "invalid",
    options: "invalid",
  });
});
test("stored permission subjects retain actual command and cwd rather than invented details", async () => {
  const question = {
    sessionId: "native",
    title: "Run command?",
    subject: {
      type: "command",
      command: "printf 'exact'",
      cwd: "/workspace/project",
    },
    options: [{ optionId: "once", name: "Allow once", kind: "allow_once" }],
  };
  const api = apiFor([question]);
  api.fields = async () => ({
    fields: [
      {
        field: RecordField.RECORD_FIELD_REQUEST,
        updatedOrdinal: 1n,
        totalBytes: BigInt(Buffer.byteLength(JSON.stringify(question) + "\n")),
      },
    ],
  });
  api.requestAnswer = async () => ({});
  const record = await readConversationRecord(
    api,
    header,
    metadata(RecordKind.RECORD_KIND_REQUEST, "occurrence"),
  );
  assert.equal(record.question.content[0].text, question.subject.command);
  assert.equal(record.question.locations[0].path, question.subject.cwd);
  assert.equal(record.responder, undefined);
});
