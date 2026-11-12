import type { ConversationRecord } from "@chapchoo/web-core/conversation-types";
function record(
  kind: ConversationRecord["kind"],
  id: string,
  text: string,
  ordinal: bigint,
): ConversationRecord {
  return {
    key: `${kind}:${id}`,
    nativeId: id,
    kind,
    anchor: { ordinal, index: 0n },
    updatedOrdinal: ordinal,
    content: text ? [{ type: "text", text }] : [],
    locations: [],
  };
}
/** Disposable design fixtures; no real execution or filesystem access. */
export const conversationReviewRecords: ConversationRecord[] = [
  record(
    "user",
    "draft-human",
    "채팅의 **색상 설정**을 확인하고, 코드 정리 후 테스트해 줘.",
    1n,
  ),
  {
    ...record("assistant", "draft-agent", "", 2n),
    content: [
      { type: "text", text: "현재 **테마 설정**을 확인하고 " },
      {
        type: "text",
        text: "중복된 스타일을 정리하겠습니다.\n\n- 색상 변수 확인\n- 스타일 수정\n- 테스트 실행\n\n[검증 기준](https://example.com/checks)도 함께 확인합니다.",
      },
    ],
  },
  {
    ...record(
      "tool",
      "draft-read",
      "1  :root {\n2    --color-surface: #20271e;\n3    --color-ink: #ecefe7;\n4    --color-muted: #a9b2a5;\n5  }\n6",
      3n,
    ),
    toolKind: "read",
    title: "테마 파일 읽기",
    status: "completed",
    locations: [{ path: "src/theme.css" }],
    toolInput: { json: '{"path":"src/theme.css","offset":0,"limit":80}' },
  },
  {
    ...record("tool", "draft-search", "src/theme.css:12\nsrc/chat.css:48", 4n),
    toolKind: "search",
    title: "색상 변수 사용처 검색",
    status: "completed",
    toolInput: { json: '{"query":"--color-surface","path":"src/"}' },
  },
  {
    ...record("tool", "draft-edit", "", 5n),
    toolKind: "edit",
    title: "배경색 토큰으로 통일",
    status: "completed",
    locations: [{ path: "src/chat.css" }],
    content: [
      {
        type: "diff",
        path: "src/chat.css",
        oldText: "background: #20271e;",
        newText: "background: var(--color-surface);\ncolor: var(--color-ink);",
      },
    ],
  },
  {
    ...record(
      "tool",
      "draft-execute",
      "✓ theme.test.ts (4 tests)\n✓ chat.test.ts (8 tests)\n\nTest Files  2 passed (2)\n     Tests  12 passed (12)",
      6n,
    ),
    toolKind: "execute",
    title: "테스트 실행",
    status: "completed",
    toolInput: {
      json: '{"command":"pnpm test --filter chat","cwd":"/workspace/app"}',
    },
  },
  {
    ...record(
      "tool",
      "draft-failed",
      'Error: no script named "test:visual"\nRun pnpm run to list available scripts.',
      7n,
    ),
    toolKind: "execute",
    title: "시각 테스트 실행",
    status: "failed",
    toolInput: { json: '{"command":"pnpm test:visual"}' },
  },
  {
    ...record("tool", "draft-fetch", "응답을 기다리고 있습니다.", 8n),
    toolKind: "fetch",
    title: "참고 문서 확인",
    status: "in_progress",
    toolInput: { json: '{"url":"https://example.com/theme-guidelines"}' },
  },
  {
    ...record(
      "assistant",
      "draft-final",
      "테마의 **색상 변수**로 정리했고, 단위 테스트 12개가 통과했습니다.\n\n시각 테스트는 등록된 명령이 없어 실행하지 못했습니다.\n\n```sh\npnpm test --filter chat\n```",
      9n,
    ),
  },
  {
    ...record("tool", "draft-publication", "", 10n),
    toolKind: "execute",
    title: "mcp.chapchoo.send_message",
    status: "completed",
    toolInput: {
      json: JSON.stringify({
        server: "chapchoo",
        tool: "send_message",
        arguments: {
          text: "테마의 **색상 변수**를 정리했습니다.\n\n루트에서 `pnpm test`를 실행하실 수 있습니다.",
        },
      }),
    },
    toolOutput: {
      json: '{"result":{"content":[{"type":"text","text":"Message accepted."}],"structuredContent":{"message_id":"fixture-message"}},"error":null}',
    },
  },
];
