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
import { TypedAnswer } from "@chapchoo/rpc-client/conversations";
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
