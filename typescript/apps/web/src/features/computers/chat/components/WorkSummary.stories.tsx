import { Button, Group, Stack, TextInput } from "@mantine/core";
import { useState } from "react";
import { chatCopy } from "../copy";
import { WorkSummary } from "./WorkSummary";
import type { ConversationWorkSummary } from "@chapchoo/web-core/conversation-types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const fixture: ConversationWorkSummary = {
  id: "work",
  title: "Review conversation fixture",
  waitingForInput: false,
  tasks: [
    { text: "Read the instructions", status: "completed" },
    { text: "Review source changes", status: "inProgress" },
    { text: "Summarize remaining work", status: "pending" },
  ],
};

function Fixture({ initial = true }: { initial?: boolean }): ReactElement {
  const [work, setWork] = useState(initial ? fixture : void 0);
  return (
    <Stack p="md" h="100dvh" justify="space-between">
      <Group>
        <Button onClick={() => setWork(fixture)}>Show work</Button>
        <Button onClick={() => setWork({ ...fixture, waitingForInput: true })}>
          Ask for input
        </Button>
        <Button onClick={() => setWork(void 0)}>Finish work</Button>
      </Group>
      <footer>
        <WorkSummary work={work} copy={chatCopy("en").workSummary} />
        <TextInput label="Message" />
      </footer>
    </Stack>
  );
}

const meta: Meta<typeof Fixture> = {
  title: "Chat/Work Summary",
  component: Fixture,
  parameters: { layout: "fullscreen" },
};
export default meta;
type Story = StoryObj<typeof meta>;
export const Initial: Story = {};
export const Live: Story = { args: { initial: false } };
