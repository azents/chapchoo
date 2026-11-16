import { Button, Group } from "@mantine/core";
import { useState } from "react";
import { chatCopy } from "../copy";
import { ReturnToLatest } from "../ReturnToLatest";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture({
  initialUnread = false,
  locale = "ko-KR",
}: {
  initialUnread?: boolean;
  locale?: string;
}): ReactElement {
  const [visible, setVisible] = useState(true);
  const [unread, setUnread] = useState(initialUnread);
  const copy = chatCopy(locale);
  return (
    <div style={{ padding: "2rem" }}>
      <Group>
        <Button onClick={() => setVisible(true)}>Read earlier</Button>
        <Button onClick={() => setUnread(true)}>Receive message</Button>
        <Button
          onClick={() => {
            setVisible(false);
            setUnread(false);
          }}
        >
          Reach bottom
        </Button>
      </Group>
      <div style={{ position: "relative", marginTop: "8rem" }}>
        <ReturnToLatest
          visible={visible}
          unread={unread}
          label={copy.scrollDown}
          unreadLabel={copy.newMessages}
          onLatest={() => {
            setVisible(false);
            setUnread(false);
          }}
        />
      </div>
    </div>
  );
}
const meta: Meta<typeof Fixture> = {
  title: "Chat/Return to latest",
  component: Fixture,
};
export default meta;
type Story = StoryObj<typeof meta>;
export const Reading: Story = {};
export const Unread: Story = { args: { initialUnread: true } };
export const LongFrenchLabel: Story = {
  args: { initialUnread: true, locale: "fr-FR" },
};
