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

/** Static UI review data; no execution or provider requests. */
export const conversationReviewRecords: ConversationRecord[] = [
  record(
    "user",
    "review-user",
    "서울에서 보낼 **조용한 오후 일정**을 정리해 주세요.\n\n- 공원 산책\n- 카페에서 휴식",
    1n,
  ),
  record(
    "assistant",
    "review-assistant",
    "공원을 산책하고 근처 카페에서 쉬는 일정입니다.\n\n**추천 순서**\n1. 공원 산책\n2. 카페에서 휴식\n\n[참고 링크](https://example.com)\n\n| 장소 | 시간 |\n| --- | --- |\n| 공원 | 14:00 |\n| 카페 | 15:00 |\n\n```text\n오후 일정 정리 완료\n```\n\n> 운영 시간은 방문 전에 확인해 주세요.",
    2n,
  ),
  {
    ...record(
      "thought",
      "review-thought",
      "이동 거리를 줄이고 여유 시간을 확보하겠습니다.",
      3n,
    ),
    title: "경로 비교",
    status: "completed",
    description: "두 장소 사이의 이동 경로를 비교했습니다.",
  },
  {
    ...record("plan", "review-plan", "", 4n),
    title: "오후 일정",
    plan: {
      type: "items",
      items: [
        { content: "공원 운영 시간 확인", status: "completed" },
        { content: "카페 영업 시간 확인", status: "in_progress" },
        { content: "최종 일정 정리", status: "pending" },
      ],
    },
  },
  {
    ...record("plan", "review-text-plan", "", 5n),
    title: "텍스트 계획",
    plan: {
      type: "text",
      text: "**먼저** 운영 시간을 확인하고, 이후 이동 경로를 정리합니다.",
    },
  },
  {
    ...record("tool", "review-tool", "조회한 결과를 정리했습니다.", 6n),
    title: "운영 시간 조회",
    status: "completed",
    toolInput: { json: '{"query":"서울 공원 운영 시간"}' },
    toolOutput: { json: '{"opens":"09:00","closes":"18:00"}' },
    locations: [{ path: "/workspace/itinerary.md", line: 3 }],
  },
  {
    ...record("tool", "review-diff", "", 7n),
    title: "일정 파일 수정",
    status: "completed",
    content: [
      {
        type: "diff",
        path: "itinerary.md",
        oldText: "14:00 미정",
        newText: "14:00 공원 산책",
      },
      {
        type: "resource",
        title: "저장한 일정",
        uri: "file:///workspace/itinerary.md",
        text: "14:00 공원 산책\n15:00 카페 휴식",
      },
    ],
  },
  {
    ...record("terminal", "review-terminal", "", 8n),
    title: "일정 확인 명령",
    terminal: {
      text: "itinerary.md: OK\n",
      base64: "aXRpbmVyYXJ5Lm1kOiBPSwo=",
      exitCode: 0,
    },
  },
  {
    ...record("notice", "review-notice", "일정 파일을 저장했습니다.", 9n),
    title: "저장 완료",
  },
  {
    ...record(
      "state",
      "review-state",
      "작업이 완료되어 다음 메시지를 기다립니다.",
      10n,
    ),
    title: "대기",
    status: "completed",
  },
  {
    ...record("request", "review-permission", "", 11n),
    question: {
      type: "permission",
      title: "이 명령 실행을 허용할까요?",
      description: "일정 파일의 내용을 확인합니다.",
      content: [{ type: "text", text: "cat itinerary.md" }],
      locations: [{ path: "/workspace/itinerary.md", line: 1 }],
      options: [
        { id: "once", title: "이번만 허용", scope: "This command only" },
        { id: "session", title: "세션 동안 허용", scope: "Current session" },
        { id: "reject", title: "거부", scope: "This command only" },
      ],
    },
    responder: { token: "static-review-only" },
  },
  {
    ...record("assistant", "review-media", "", 12n),
    content: [
      { type: "text", text: "이미지·오디오 표시 예시입니다." },
      {
        type: "image",
        mimeType: "image/png",
        data: "iVBORw0KGgoAAAANSUhEUgAAACAAAAASCAIAAAC1qksFAAAAIklEQVR4nGOsWDGZgZaAiaamj1owasGoBaMWjFowagEUAAAVnwHX+CEdHwAAAABJRU5ErkJggg==",
      },
      {
        type: "audio",
        mimeType: "audio/wav",
        data: "UklGRiQAAABXQVZFZm10IBAAAAABAAEAQB8AAIA+AAACABAAZGF0YQAAAAA=",
      },
      { type: "unsupported", kind: "지원하지 않는 콘텐츠" },
    ],
  },
  {
    ...record("request", "review-form", "", 13n),
    question: {
      type: "form",
      title: "일정 조건을 알려 주세요",
      schemaTitle: "오후 일정",
      fields: [
        {
          name: "note",
          title: "요청사항",
          required: true,
          type: "text",
          minLength: 1,
          maxLength: 100,
        },
        {
          name: "place",
          title: "장소",
          required: true,
          type: "text",
          choices: [
            { title: "공원", value: "park" },
            { title: "카페", value: "cafe" },
          ],
        },
        { name: "walk", title: "도보 이동", required: true, type: "boolean" },
        {
          name: "stops",
          title: "방문 장소",
          required: false,
          type: "strings",
          choices: [
            { title: "공원", value: "park" },
            { title: "카페", value: "cafe" },
          ],
          minItems: 1,
          maxItems: 2,
        },
        {
          name: "minutes",
          title: "이동 시간",
          required: false,
          type: "integer",
          minimum: 0,
          maximum: 60,
        },
        {
          name: "budget",
          title: "예산 비율",
          required: false,
          type: "number",
          minimum: 0,
          maximum: 1,
        },
        {
          name: "unsupported",
          title: "추가 입력",
          required: false,
          type: "unsupported",
        },
      ],
    },
    responder: { token: "static-form-only" },
  },
  {
    ...record("request", "review-unsupported", "", 14n),
    question: { type: "unsupported", title: "지원하지 않는 질문" },
  },
];
