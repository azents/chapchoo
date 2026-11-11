import {
  initialQuestionDraft,
  reviewQuestionAnswer,
} from "@chapchoo/web-core/conversation-form";
import { Box } from "@mantine/core";
import { useState } from "react";
import { chatCopy } from "../copy";
import { ConversationBody } from "./ConversationBody";
import { conversationReviewRecords } from "./conversationReviewFixture";
import type { ConversationControls } from "../types";
import type { QuestionDraft } from "@chapchoo/web-core/conversation-form";
import type {
  ConversationAnswer,
  ConversationRecord,
  Question,
} from "@chapchoo/web-core/conversation-types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const formQuestion: Question = {
  type: "form",
  title: "Review the deployment values",
  description: "All choices stay expanded for review.",
  fields: [
    {
      name: "channel",
      title: "Release channel",
      description: "Choose the intended audience.",
      required: true,
      type: "text",
      choices: [
        { title: "Internal preview", value: "internal-preview" },
        { title: "Public release", value: "public-release" },
      ],
    },
    {
      name: "enabled",
      title: "Enable notifications",
      required: true,
      type: "boolean",
      default: false,
    },
    {
      name: "note",
      title: "Release note",
      required: true,
      type: "text",
      default: "",
    },
    {
      name: "targets",
      title: "Targets",
      required: false,
      type: "strings",
      choices: [
        { title: "Development", value: "dev" },
        { title: "Production", value: "prod" },
      ],
    },
    {
      name: "limit",
      title: "Exact limit",
      required: true,
      type: "integer",
      default: 9223372036854775807n,
    },
    {
      name: "owner",
      title: "Contact email",
      required: false,
      type: "text",
      format: "email",
    },
    {
      name: "ratio",
      title: "Review ratio",
      required: false,
      type: "number",
      minimum: 0,
      maximum: 1,
      default: 0.5,
    },
    {
      name: "external",
      title: "Unsupported field",
      required: false,
      type: "unsupported",
    },
  ],
};
const permissionQuestion: Question = {
  type: "permission",
  title: "Run the supplied command?",
  description: "The native request provided these choices.",
  content: [
    {
      type: "text",
      text: "printf '%s\\n' 'review this complete command'\ncat ./notes/release.txt",
    },
  ],
  locations: [{ path: "/workspace/current-project" }],
  options: [
    { id: "allow-once:exact", title: "Allow once", scope: "This command only" },
    {
      id: "allow-session:exact",
      title: "Allow for this session",
      scope: "Current session",
    },
    { id: "reject-once:exact", title: "Reject", scope: "This command only" },
  ],
};
function record(
  kind: ConversationRecord["kind"],
  id: string,
  ordinal: bigint,
  content: ConversationRecord["content"],
): ConversationRecord {
  return {
    key: `${kind}:${id}`,
    kind,
    nativeId: id,
    anchor: { ordinal, index: 0n },
    updatedOrdinal: ordinal,
    content,
    locations: [],
  };
}
function Fixture({
  mode,
  locale,
  surface,
}: {
  mode:
    | "form"
    | "permission"
    | "retained"
    | "unavailable"
    | "uncertain"
    | "retainedUncertain"
    | "validationRejected"
    | "loading"
    | "error"
    | "review"
    | "media"
    | "inputs"
    | "captured"
    | "content";
  locale: string;
  surface: "transcript" | "history";
}): ReactElement {
  const copy = chatCopy(locale);
  const question = mode === "permission" ? permissionQuestion : formQuestion;
  const request: ConversationRecord = {
    ...record("request", "question", 3n, []),
    question,
    responder: mode === "unavailable" ? void 0 : { token: "fixture-only" },
  };
  const retained: ConversationAnswer = {
    type: "formAccept",
    content: {
      channel: { type: "text", value: "public-release" },
      enabled: { type: "boolean", value: false },
      note: { type: "text", value: "" },
      limit: { type: "integer", value: 9223372036854775807n },
      owner: { type: "text", value: "owner@example.com" },
    },
  };
  if (mode === "retained" || mode === "retainedUncertain") {
    request.answer = { value: retained, delivery: "delivered" };
  }
  const [draft, setDraft] = useState<QuestionDraft>({
    ...initialQuestionDraft(question),
    state:
      mode === "uncertain" || mode === "retainedUncertain"
        ? "uncertain"
        : mode === "validationRejected"
          ? "invalid"
          : "editing",
  });
  const records =
    mode === "review"
      ? conversationReviewRecords
      : mode === "media"
        ? [
            record("assistant", "media", 1n, [
              {
                type: "text",
                text: "이미지·오디오·리소스·지원하지 않는 콘텐츠의 표시 예시입니다.",
              },
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
              {
                type: "resource",
                title: "일정 문서",
                uri: "file:///workspace/itinerary.md",
                text: "공원 산책 → 카페 휴식",
              },
              { type: "unsupported", kind: "지원하지 않는 콘텐츠" },
            ]),
          ]
        : mode === "inputs"
          ? []
          : mode === "captured"
            ? [
                record("user", "native-user", 1n, [
                  { type: "text", text: "ㅎㅇ" },
                ]),
              ]
            : mode === "content"
              ? [
                  record("user", "user", 1n, [
                    {
                      type: "text",
                      text: "**User Markdown** with [a user reference](https://example.com/user).\n\n- First request\n- Second request\n\n`inline code`\n\n<script>alert('not executable')</script>",
                    },
                  ]),
                  record("assistant", "assistant-one", 2n, [
                    {
                      type: "text",
                      text: "## First utterance\n\nA **streamed** answer with [a reference](https://example.com).\n",
                    },
                    {
                      type: "text",
                      text: "\n| Item | Value |\n| --- | --- |\n| Complete | Untruncated |",
                    },
                  ]),
                  record("assistant", "assistant-two", 3n, [
                    { type: "text", text: "A distinct assistant utterance." },
                  ]),
                  {
                    ...record("tool", "tool", 4n, [
                      {
                        type: "diff",
                        path: "/workspace/notes.txt",
                        oldText: "previous line",
                        newText: "new line",
                      },
                    ]),
                    title: "Update notes",
                    status: "completed",
                  },
                  {
                    ...record("tool", "raw-tool", 5n, []),
                    title: "Read note",
                    status: "completed",
                    toolInput: {
                      json: '{"path":"notes.txt","limit":9223372036854775807}',
                    },
                    toolOutput: {
                      json: '{"lines":["complete tool-only result"],"enabled":false}',
                    },
                  },
                  {
                    ...record("terminal", "terminal", 5n, []),
                    title: "printf",
                    terminal: {
                      text: "Complete output\n",
                      base64: "Q29tcGxldGUgb3V0cHV0Cg==",
                      exitCode: 0,
                    },
                  },
                ]
              : [request];
  const controls: ConversationControls = {
    region: {
      header: {
        agentId: "agent",
        conversationId: "conversation",
        state: mode === "loading" ? "loading" : "ready",
        source: {
          agentId: "agent",
          conversationId: "conversation",
          snapshotId: "snapshot",
        },
        foreground: "requiresAction",
      },
      records: mode === "loading" ? [] : records,
      inputs:
        mode === "captured"
          ? [
              {
                id: "unmatched-failed-input",
                requestId: "unmatched-failed",
                conversationId: "conversation",
                acceptedAt: 2n,
                text: "전달되지 않은 메시지",
                observation: "failed",
              },
            ]
          : [],
      renderedAt: 1,
    },
    read: mode === "loading" ? "loading" : mode === "error" ? "error" : "ready",
    readingOlder: false,
    inputs:
      mode === "inputs"
        ? [
            {
              requestId: "sending",
              text: "전송 중인 메시지",
              observation: "sending",
            },
            {
              requestId: "preparing",
              text: "대화 준비 중인 메시지",
              observation: "preparing",
            },
            {
              requestId: "accepted",
              text: "접수된 메시지",
              observation: "accepted",
            },
            {
              requestId: "failed",
              text: "**확인**",
              observation: "failed",
            },
            {
              requestId: "uncertain",
              text: "**확인**",
              observation: "uncertain",
            },
          ]
        : [],
    stop: "idle",
    drafts: { [request.key]: draft },
    onRead: () => {},
    onOlder: () => {},
    onLatest: () => {},
    onHistory: () => {},
    onQuestionOption: (_record, optionId) =>
      setDraft((value) => ({
        ...value,
        optionId,
        errors: {},
        state: "editing",
      })),
    onQuestionField: (_record, name, value) =>
      setDraft((current) => {
        const fields = { ...current.fields };
        if (value === null) {
          delete fields[name];
        }
        return {
          ...current,
          fields: value === null ? fields : { ...fields, [name]: value },
          errors: {},
          state: "editing",
        };
      }),
    onAnswer: (_record, explicit) => {
      const reviewed = explicit
        ? { answer: explicit, errors: {} }
        : reviewQuestionAnswer(question, draft);
      setDraft((value) => ({
        ...value,
        errors: reviewed.errors,
        state: reviewed.answer ? "accepted" : "editing",
        answer: reviewed.answer,
      }));
    },
  };
  return (
    <Box maw="49rem" p="md" mx="auto">
      <ConversationBody
        controls={controls}
        copy={copy}
        writable={mode !== "unavailable" && surface !== "history"}
        agentName="Corgi"
        agentProfile={{ basePath: "", assetKey: "03-corgi" }}
        surface={surface}
      />
    </Box>
  );
}
const meta = {
  title: "Chat/Conversation body",
  component: Fixture,
  args: { mode: "form", locale: "en-US", surface: "transcript" },
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Form: Story = {};
export const Permission: Story = { args: { mode: "permission" } };
export const Retained: Story = { args: { mode: "retained" } };
export const Unavailable: Story = { args: { mode: "unavailable" } };
export const Uncertain: Story = { args: { mode: "uncertain" } };
export const RetainedAfterUncertainty: Story = {
  args: { mode: "retainedUncertain" },
};
export const ValidationRejected: Story = {
  args: { mode: "validationRejected" },
};
export const Loading: Story = { args: { mode: "loading" } };
export const Error: Story = { args: { mode: "error" } };
export const Content: Story = { args: { mode: "content" } };
export const HistoryContent: Story = {
  args: { mode: "content", surface: "history" },
};
export const HistoryForm: Story = { args: { surface: "history" } };
export const HistoryPermission: Story = {
  args: { mode: "permission", surface: "history" },
};
export const ReviewTranscript: Story = {
  args: { mode: "review", locale: "ko-KR" },
};
export const ReviewHistory: Story = {
  args: { mode: "review", surface: "history", locale: "ko-KR" },
};
export const Media: Story = { args: { mode: "media", locale: "ko-KR" } };
export const InputStates: Story = { args: { mode: "inputs", locale: "ko-KR" } };
export const CapturedHistory: Story = {
  args: { mode: "captured", locale: "ko-KR", surface: "history" },
};
export const Korean: Story = { args: { locale: "ko-KR" } };
export const Japanese: Story = { args: { locale: "ja-JP" } };
export const French: Story = { args: { locale: "fr-FR" } };
