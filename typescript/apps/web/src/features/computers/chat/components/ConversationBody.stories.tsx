import {
  initialQuestionDraft,
  reviewQuestionAnswer,
} from "@chapchoo/web-core/conversation-form";
import { Box } from "@mantine/core";
import { useState } from "react";
import { chatCopy } from "../copy";
import { ConversationBody } from "./ConversationBody";
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
}: {
  mode:
    | "form"
    | "permission"
    | "retained"
    | "unavailable"
    | "uncertain"
    | "loading"
    | "error"
    | "content";
  locale: string;
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
  if (mode === "retained") {
    request.answer = { value: retained, delivery: "delivered" };
  }
  const [draft, setDraft] = useState<QuestionDraft>({
    ...initialQuestionDraft(question),
    state: mode === "uncertain" ? "uncertain" : "editing",
  });
  const records =
    mode === "content"
      ? [
          record("user", "user", 1n, [
            {
              type: "text",
              text: "  **This is plain user text**, not Markdown.\nNext line  ",
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
      renderedAt: 1,
    },
    read: mode === "loading" ? "loading" : mode === "error" ? "error" : "ready",
    readingOlder: false,
    inputs: [],
    stop: "idle",
    drafts: { [request.key]: draft },
    onRead: () => {},
    onOlder: () => {},
    onLatest: () => {},
    onHistory: () => {},
    onQuestionOption: (_record, optionId) =>
      setDraft((value) => ({ ...value, optionId, errors: {} })),
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
        writable={mode !== "unavailable"}
        agentName="Corgi"
      />
    </Box>
  );
}
const meta = {
  title: "Chat/Conversation body",
  component: Fixture,
  args: { mode: "form", locale: "en-US" },
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Form: Story = {};
export const Permission: Story = { args: { mode: "permission" } };
export const Retained: Story = { args: { mode: "retained" } };
export const Unavailable: Story = { args: { mode: "unavailable" } };
export const Uncertain: Story = { args: { mode: "uncertain" } };
export const Loading: Story = { args: { mode: "loading" } };
export const Error: Story = { args: { mode: "error" } };
export const Content: Story = { args: { mode: "content" } };
export const Korean: Story = { args: { locale: "ko-KR" } };
export const Japanese: Story = { args: { locale: "ja-JP" } };
export const French: Story = { args: { locale: "fr-FR" } };
