import { Button, Modal, Stack, Text } from "@mantine/core";
import { useRef, useState } from "react";
import { Sheet, SheetViewport } from "./Sheet";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture({ fadeTop = false }: { fadeTop?: boolean }): ReactElement {
  const [opened, setOpened] = useState(true);
  const [confirming, setConfirming] = useState(false);
  const confirmRef = useRef<HTMLButtonElement>(null);
  return (
    <>
      <Button onClick={() => setOpened(true)}>Open sheet</Button>
      <Sheet
        opened={opened}
        onClose={() => setOpened(false)}
        title="History"
        size={fadeTop ? "78dvh" : "100dvh"}
        closeButtonProps={{ "aria-label": "Close sheet" }}
        trapFocus={!confirming}
        closeOnEscape={!confirming}
        closeOnClickOutside={!confirming}
      >
        <SheetViewport role="log" aria-label="Records" fadeTop={fadeTop}>
          <Stack p="lg">
            <Button ref={confirmRef} onClick={() => setConfirming(true)}>
              Open confirmation
            </Button>
            {Array.from({ length: 24 }, (_, index) => (
              <Text key={index}>
                Record {index + 1}: retained detail stays inside the body
                viewport.
              </Text>
            ))}
          </Stack>
        </SheetViewport>
      </Sheet>
      <Modal
        opened={confirming}
        onClose={() => setConfirming(false)}
        title="Confirmation"
        returnFocus={false}
        onExitTransitionEnd={() =>
          confirmRef.current?.focus({ preventScroll: true })
        }
      >
        <Button onClick={() => setConfirming(false)}>Back to sheet</Button>
      </Modal>
    </>
  );
}
const meta = { title: "Shared/Sheet", component: Fixture } satisfies Meta<
  typeof Fixture
>;
export default meta;
type Story = StoryObj<typeof meta>;
export const FullHistory: Story = { parameters: { layout: "padded" } };
export const ActivityFade: Story = { args: { fadeTop: true } };
