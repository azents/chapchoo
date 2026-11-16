"use client";
import { useState } from "react";
import { systemPromptCopy } from "../systemPromptCopy";
import { SystemPrompt } from "./SystemPrompt";
import type {
  SystemPromptEditor,
  SystemPromptProps,
} from "../systemPromptTypes";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Computers/System prompt",
  component: SystemPrompt,
  args: {
    copy: systemPromptCopy("en"),
    mobile: false,
    unavailable: false,
    editor: { type: "closed" },
    onOpen: (): void => {},
    onClose: (): void => {},
    onChange: (): void => {},
    onSave: (): void => {},
    onRead: (): void => {},
  },
} satisfies Meta<typeof SystemPrompt>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Closed: Story = {};
export const Editing: Story = {
  args: {
    editor: {
      type: "editing",
      content: "Keep responses concise.\nPreserve original source text.",
      error: null,
    },
  },
};
export const Empty: Story = {
  args: { editor: { type: "editing", content: "", error: null } },
};
export const Loading: Story = { args: { editor: { type: "loading" } } };
export const ReadFailure: Story = { args: { editor: { type: "readError" } } };
export const Saving: Story = {
  args: { editor: { type: "saving", content: "Saved draft" } },
};
export const SaveFailure: Story = {
  args: {
    editor: {
      type: "editing",
      content: "Retained draft",
      error: systemPromptCopy("en").saveFailed,
    },
  },
};
export const Mobile: Story = {
  args: {
    mobile: true,
    copy: systemPromptCopy("ko"),
    editor: {
      type: "editing",
      content: "답변은 간결하게 작성합니다.\n원문은 그대로 보존합니다.",
      error: null,
    },
  },
};
function Interactive(props: SystemPromptProps): ReactElement {
  const [saved, setSaved] = useState("Original instructions");
  const [editor, setEditor] = useState<SystemPromptEditor>({ type: "closed" });
  return (
    <SystemPrompt
      {...props}
      editor={editor}
      onOpen={() => setEditor({ type: "editing", content: saved, error: null })}
      onRead={() => setEditor({ type: "editing", content: saved, error: null })}
      onClose={() => setEditor({ type: "closed" })}
      onChange={(content) =>
        setEditor({ type: "editing", content, error: null })
      }
      onSave={() => {
        if (editor.type !== "editing") {
          return;
        }
        if (editor.content === "fail") {
          setEditor({ ...editor, error: props.copy.saveFailed });
        } else {
          setSaved(editor.content);
          setEditor({ type: "closed" });
        }
      }}
    />
  );
}
export const InteractiveEditor: Story = {
  render: (props) => <Interactive {...props} />,
};
