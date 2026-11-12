import assert from "node:assert/strict";
import { test } from "node:test";
import {
  contentText,
  plainPreview,
  recordInput,
  recordPreview,
  recordPublication,
  toolKind,
} from "./recordPresentation.ts";
import type { ConversationRecord } from "@chapchoo/web-core/conversation-types";
function record(): ConversationRecord {
  return {
    kind: "assistant",
    key: "a",
    nativeId: "a",
    anchor: { ordinal: 1n, index: 0n },
    updatedOrdinal: 1n,
    content: [],
    locations: [],
  };
}
void test("publication uses actual structured MCP identity and the normalized message body", () => {
  const body = "  **확인한 방법**입니다.\n\n`pnpm test`를 실행하세요. ";
  const original = JSON.stringify({
    server: "chapchoo",
    tool: "publish_text",
    arguments: { text: body },
  });
  const publication: ConversationRecord = {
    ...record(),
    kind: "tool",
    toolKind: "execute",
    title: "Not the body",
    content: [{ type: "text", text: "Unrelated native progress" }],
    toolInput: { json: original },
    toolOutput: {
      json: JSON.stringify({
        result: {
          content: [{ type: "text", text: "Message accepted." }],
          structuredContent: { message_id: "message" },
        },
        error: null,
      }),
    },
  };
  assert.equal(recordPreview(publication), plainPreview(body));
  assert.equal(
    recordPreview({
      ...record(),
      kind: "user",
      content: [{ type: "text", text: body }],
    }),
    recordPreview(publication),
  );
  assert.deepEqual(recordPublication(publication), {
    text: body,
    resultText: "Message accepted.",
  });
  assert.equal(publication.toolInput?.json, original);
  assert.equal(
    recordPublication({ ...publication, toolOutput: void 0 })?.text,
    body,
  );
  assert.equal(
    recordPublication({ ...publication, toolOutput: { json: "{" } })?.text,
    body,
  );
  for (const candidate of [
    { ...publication, kind: "assistant" as const },
    {
      ...publication,
      toolKind: "other",
      toolInput: { json: '{"text":"looks like a message"}' },
    },
    {
      ...publication,
      title: "mcp.chapchoo.publish_text",
      toolInput: { json: "{}" },
    },
    {
      ...publication,
      toolInput: {
        json: '{"server":"another","tool":"publish_text","arguments":{"text":"same"}}',
      },
    },
    {
      ...publication,
      toolInput: {
        json: '{"server":"chapchoo","tool":"other","arguments":{"text":"same"}}',
      },
    },
  ]) {
    assert.equal(recordPublication(candidate), null);
  }
});

void test("message preview preserves native chunk whitespace and strips actual Markdown syntax", () => {
  const r = record();
  r.content = [
    { type: "text", text: "정상" },
    {
      type: "text",
      text: "적으로 **응답** 중입니다.\n\n[검증 기준](https://example.com)\n\n- 첫째\n- 둘째\n\n<!-- private -->",
    },
  ];
  assert.equal(
    recordPreview(r),
    "정상적으로 응답 중입니다. 검증 기준 첫째 둘째",
  );
  assert.equal(
    plainPreview("`file_name.ts` &amp; **강조** ~~삭제~~ ![그림](x)"),
    "file_name.ts & 강조 삭제 그림",
  );
  assert.equal(
    contentText([
      { type: "text", text: "ab" },
      { type: "text", text: " cd" },
    ]),
    "ab cd",
  );
});
void test("preview stays Unicode-safe and treats non-text content as a boundary", () => {
  const text = plainPreview("😀".repeat(200));
  assert.equal(Array.from(text).length, 180);
  assert.equal(text, "😀".repeat(180));
  assert.equal(
    plainPreview(
      contentText([
        { type: "text", text: "one" },
        { type: "image", mimeType: "image/png", data: "" },
        { type: "text", text: "two" },
      ]),
    ),
    "one two",
  );
});
void test("ACP kind controls subjects; unknown kinds and malformed raw data remain readable", () => {
  const r: ConversationRecord = {
    ...record(),
    kind: "tool",
    toolKind: "execute",
    title: "Run checks",
    toolInput: { json: '{"command":"pnpm test","cwd":"/app"}' },
  };
  assert.equal(toolKind("execute"), "execute");
  assert.equal(toolKind("_custom"), "other");
  assert.equal(recordPreview(r), "pnpm test");
  assert.equal(
    recordPreview({
      ...r,
      toolKind: "read",
      locations: [{ path: "src/a.ts", line: 2 }],
    }),
    "src/a.ts",
  );
  assert.equal(
    recordPreview({
      ...r,
      toolKind: "search",
      toolInput: { json: '{"query":"hello"}' },
    }),
    "hello",
  );
  assert.equal(
    recordPreview({
      ...r,
      toolKind: "move",
      toolInput: { json: '{"source":"a","destination":"b"}' },
    }),
    "a → b",
  );
  assert.deepEqual(recordInput({ ...r, toolInput: { json: "{" } }), {});
  assert.equal(
    recordPreview({ ...r, toolKind: "_custom", toolInput: { json: "{}" } }),
    "Run checks",
  );
});

void test("request previews show retained question context before repeating its title", () => {
  const r: ConversationRecord = {
    ...record(),
    kind: "request",
    question: {
      type: "form",
      title: "Release details",
      description: "Choose the **release target**.",
      fields: [],
    },
  };
  assert.equal(recordPreview(r), "Choose the release target.");
  assert.equal(
    recordPreview({
      ...r,
      question: {
        type: "permission",
        title: "Run a command?",
        options: [],
        locations: [],
        content: [{ type: "text", text: "`pnpm test`" }],
      },
    }),
    "pnpm test",
  );
  assert.equal(
    recordPreview({
      ...r,
      question: { type: "unsupported", title: "Unknown request" },
    }),
    "Unknown request",
  );
});
