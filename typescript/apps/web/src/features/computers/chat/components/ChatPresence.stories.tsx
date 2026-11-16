import { Button, Group, Stack, Text } from "@mantine/core";
import { useState } from "react";
import { ChatPresence } from "../ChatPresence";
import { ThinkingIndicator } from "./ThinkingIndicator";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture(): ReactElement {
  const [shown, setShown] = useState(false);
  const [older, setOlder] = useState(false);
  const [initialShown, setInitialShown] = useState(true);
  const [expanded, setExpanded] = useState(false);
  const [source, setSource] = useState(0);
  return (
    <Stack maw="40rem" mx="auto" p="xl">
      <Group>
        <Button onClick={() => setShown((value) => !value)}>Toggle item</Button>
        <Button onClick={() => setOlder(true)}>Prepend history</Button>
        <Button onClick={() => setInitialShown(false)}>Remove initial</Button>
        <Button onClick={() => setExpanded((value) => !value)}>
          Resize item
        </Button>
        <Button
          onClick={() => {
            setSource((value) => value + 1);
            setShown(false);
          }}
        >
          Change source
        </Button>
      </Group>
      <ChatPresence key={source} spaced>
        {older ? (
          <Text key="older" data-testid="older" data-presence-order={0n}>
            Earlier message
          </Text>
        ) : null}
        {initialShown ? (
          <Text key="initial" data-testid="initial" data-presence-order={1n}>
            Initial server-rendered message
          </Text>
        ) : null}
        {shown ? (
          <Stack key="dynamic" data-testid="dynamic">
            <Text>New response request</Text>
            <Button>Answer request</Button>
            {expanded ? <Text>{"More content. ".repeat(50)}</Text> : null}
          </Stack>
        ) : null}
        {shown ? <ThinkingIndicator key="thinking" label="Thinking" /> : null}
      </ChatPresence>
    </Stack>
  );
}
const meta: Meta<typeof Fixture> = {
  title: "Chat/Presence",
  component: Fixture,
  parameters: { layout: "fullscreen" },
};
export default meta;
type Story = StoryObj<typeof meta>;
export const Lifecycle: Story = {};
