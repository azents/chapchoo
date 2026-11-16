import { useId, useRef, useState } from "react";
import { MessageEditor } from "../MessageEditor";
import classes from "./Chat.module.css";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture({
  initial = "",
  pending = false,
  writable = true,
  error = false,
  placeholder = "코기 · 메시지 보내기",
}: {
  initial?: string;
  pending?: boolean;
  writable?: boolean;
  error?: boolean;
  placeholder?: string;
}): ReactElement {
  const [value, setValue] = useState(initial);
  const id = useId();
  const editorRef = useRef<HTMLDivElement>(null);
  return (
    <>
      <div className={classes.composer}>
        <MessageEditor
          id={id}
          editorRef={editorRef}
          value={value}
          writable={writable}
          pending={pending}
          error={error}
          placeholder={placeholder}
          label="메시지"
          onChange={setValue}
          onSend={() => {}}
        />
      </div>
      <output data-draft>{JSON.stringify(value)}</output>
      {error ? <span id="chat-input-error">전송 요청 실패</span> : null}
    </>
  );
}
const meta = {
  title: "Chat/Message editor",
  component: Fixture,
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Empty: Story = { args: {} };
export const Prefilled: Story = { args: { initial: "안녕\n둘째 줄 🙂" } };
export const Pending: Story = {
  args: { initial: "전송할 원문", pending: true },
};
export const ReadOnly: Story = { args: { writable: false } };
export const Error: Story = { args: { initial: "실패한 초안", error: true } };
export const LongPlaceholder: Story = {
  args: {
    placeholder:
      "아주 긴 에이전트 이름을 가진 코기 · 메시지를 입력해서 대화를 시작해 주세요",
  },
};
