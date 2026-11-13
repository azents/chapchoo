import assert from "node:assert/strict";
import { test } from "node:test";
import {
  initialQuestionDraft,
  reviewQuestionAnswer,
} from "../dist/conversation-form.js";
import {
  decodeAnswer,
  encodeAnswer,
  readConversationRecord,
} from "../dist/conversation.js";
import {
  TypedAnswer,
  RecordKind,
  RecordField,
} from "@chapchoo/rpc-client/conversations";

test("native pattern evaluation stays at the bounded server while ordinary browser constraints remain active", () => {
  const question = {
    type: "form",
    title: "Native pattern",
    fields: [
      {
        name: "value",
        type: "text",
        title: "Value",
        required: true,
        pattern: "(",
        minLength: 2,
      },
    ],
  };
  const draft = initialQuestionDraft(question);
  draft.fields.value = { type: "text", value: "editable" };
  assert.equal(
    reviewQuestionAnswer(question, draft).answer?.type,
    "formAccept",
  );
  draft.fields.value.value = "x";
  assert.deepEqual(reviewQuestionAnswer(question, draft).errors, {
    value: "invalid",
  });
});

test("tool-only original input and output remain independent with exact integers and clear semantics", async () => {
  const payloads = new Map([
    [9, Buffer.from('{"limit":9223372036854775807,"enabled":false}\n')],
    [10, Buffer.from('["result",0]\n')],
  ]);
  const api = {
    fields: async () => ({
      fields: [...payloads].map(([field, bytes]) => ({
        field,
        updatedOrdinal: 2n,
        totalBytes: BigInt(bytes.length),
      })),
    }),
    readField: async ({ field, offset, length }) => {
      const bytes = payloads.get(field);
      return {
        content: bytes.subarray(offset, offset + length),
        offset,
        totalBytes: BigInt(bytes.length),
      };
    },
  };
  const read = () =>
    readConversationRecord(
      api,
      {
        agentId: "agent",
        state: "ready",
        source: {
          agentId: "agent",
          conversationId: "conversation",
          snapshotId: "snapshot",
        },
      },
      {
        kind: 4,
        nativeId: "raw-tool",
        anchor: { ordinal: 1n, index: 0n },
        updatedOrdinal: 2n,
      },
    );
  const record = await read();
  assert.deepEqual(record.content, []);
  assert.match(record.toolInput.json, /9223372036854775807/);
  assert.match(record.toolInput.json, /"enabled": false/);
  assert.match(record.toolOutput.json, /"result"/);
  payloads.set(9, Buffer.alloc(0));
  const cleared = await read();
  assert.equal(cleared.toolInput, undefined);
  assert.deepEqual(cleared.toolOutput, record.toolOutput);
});
test("arbitrary form keys remain own fields rather than inherited defaults or object prototype mutations", () => {
  const question = {
    type: "form",
    title: "Values",
    fields: [
      {
        name: "__proto__",
        type: "text",
        title: "Exact key",
        required: true,
        default: "",
      },
      { name: "toString", type: "boolean", title: "Enabled", required: true },
      { name: "constructor", type: "text", title: "Optional", required: false },
    ],
  };
  const draft = initialQuestionDraft(question);
  assert.equal(Object.hasOwn(draft.fields, "__proto__"), true);
  assert.equal(Object.hasOwn(draft.fields, "toString"), false);
  assert.deepEqual(reviewQuestionAnswer(question, draft).errors, {
    toString: "required",
  });
  draft.fields = {
    ...draft.fields,
    toString: { type: "boolean", value: false },
  };
  const reviewed = reviewQuestionAnswer(question, draft);
  assert.deepEqual(reviewed.errors, {});
  assert.equal(Object.hasOwn(reviewed.answer.content, "__proto__"), true);
  assert.equal(Object.hasOwn(reviewed.answer.content, "constructor"), false);
  const result = decodeAnswer(
    TypedAnswer.decode(
      TypedAnswer.encode(encodeAnswer(reviewed.answer)).finish(),
    ),
  );
  assert.deepEqual(result, reviewed.answer);
  assert.equal(Object.getPrototypeOf(result.content), Object.prototype);
});
test("invalid defaults are not fabricated into valid answers and full i64 defaults stay exact", () => {
  const question = {
    type: "form",
    title: "Defaults",
    fields: [
      {
        name: "large",
        type: "integer",
        title: "Exact",
        required: true,
        default: 9223372036854775807n,
      },
      {
        name: "choice",
        type: "text",
        title: "Choice",
        required: true,
        default: "unoffered",
        choices: [{ title: "Visible title", value: "actual-value" }],
      },
      {
        name: "empty",
        type: "text",
        title: "Empty",
        required: false,
        default: "",
      },
      {
        name: "off",
        type: "boolean",
        title: "Off",
        required: false,
        default: false,
      },
    ],
  };
  const draft = initialQuestionDraft(question);
  assert.deepEqual(draft.fields.large, {
    type: "text",
    value: "9223372036854775807",
  });
  assert.equal(draft.fields.choice, undefined);
  assert.deepEqual(draft.fields.empty, { type: "text", value: "" });
  assert.deepEqual(draft.fields.off, { type: "boolean", value: false });
  assert.deepEqual(reviewQuestionAnswer(question, draft).errors, {
    choice: "required",
  });
});
test("Notice without an optional description remains readable and invalid descriptions still fail", async () => {
  let description = null;
  const api = {
    fields: async () => ({
      fields: [
        {
          field: RecordField.RECORD_FIELD_TITLE,
          updatedOrdinal: 1n,
          totalBytes: 9n,
        },
        {
          field: RecordField.RECORD_FIELD_DESCRIPTION,
          updatedOrdinal: 1n,
          totalBytes: BigInt(
            Buffer.byteLength(JSON.stringify(description) + "\n"),
          ),
        },
      ],
    }),
    readField: async (request) => {
      const content = Buffer.from(
        JSON.stringify(
          request.field === RecordField.RECORD_FIELD_TITLE
            ? "Notice"
            : description,
        ) + "\n",
      );
      return {
        content: content.subarray(
          request.offset,
          request.offset + request.length,
        ),
        offset: request.offset,
        totalBytes: BigInt(content.length),
      };
    },
  };
  const header = {
    agentId: "agent",
    state: "ready",
    source: {
      agentId: "agent",
      conversationId: "conversation",
      snapshotId: "snapshot",
    },
  };
  const metadata = {
    kind: RecordKind.RECORD_KIND_NOTICE,
    nativeId: "notice",
    anchor: { ordinal: 1n, index: 0n },
    updatedOrdinal: 1n,
  };
  const absent = await readConversationRecord(api, header, metadata);
  assert.equal(absent.description, undefined);
  assert.equal(absent.title, "Notice");
  description = "More information";
  assert.equal(
    (await readConversationRecord(api, header, metadata)).description,
    description,
  );
  description = 42;
  await assert.rejects(readConversationRecord(api, header, metadata));
});

test("terminal output is decoded after byte assembly and retains its exact inspectable bytes", async () => {
  const bytes = Buffer.from("첫 줄\n🙂\n");
  const outputs =
    [
      JSON.stringify({ data: bytes.subarray(0, 2).toString("base64") }),
      JSON.stringify({ data: bytes.subarray(2).toString("base64") }),
    ].join("\n") + "\n";
  const exit = JSON.stringify({ exitCode: 0 }) + "\n";
  const fields = [
    {
      field: 1,
      updatedOrdinal: 2n,
      totalBytes: BigInt(Buffer.byteLength(outputs)),
    },
    {
      field: 3,
      updatedOrdinal: 2n,
      totalBytes: BigInt(Buffer.byteLength(exit)),
    },
  ];
  const api = {
    fields: async () => ({ fields }),
    readField: async (request) => {
      const content = Buffer.from(request.field === 1 ? outputs : exit);
      return {
        content: content.subarray(
          request.offset,
          request.offset + request.length,
        ),
        offset: request.offset,
        totalBytes: BigInt(content.length),
      };
    },
  };
  const record = await readConversationRecord(
    api,
    {
      agentId: "agent",
      state: "ready",
      source: {
        agentId: "agent",
        conversationId: "conversation",
        snapshotId: "snapshot",
      },
    },
    {
      kind: 6,
      nativeId: "terminal",
      anchor: { ordinal: 1n, index: 0n },
      updatedOrdinal: 2n,
    },
  );
  assert.deepEqual(record.terminal, {
    text: bytes.toString(),
    base64: bytes.toString("base64"),
    exitCode: 0,
    signal: undefined,
  });
  assert.deepEqual(record.content, []);
});
