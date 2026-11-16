import { AgentTheme } from "@chapchoo/ui/AgentTheme";
import { Box } from "@mantine/core";
import { chatCopy } from "../copy";
import { ThinkingIndicator } from "./ThinkingIndicator";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture({ locale }: { locale: string }): ReactElement {
  return (
    <AgentTheme palette="butter">
      <Box p="md">
        <ThinkingIndicator label={chatCopy(locale).thinking} />
      </Box>
    </AgentTheme>
  );
}

const meta = {
  title: "Chat/Thinking indicator",
  component: Fixture,
  args: { locale: "ko-KR" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Korean: Story = {};
export const English: Story = { args: { locale: "en-US" } };
export const Japanese: Story = { args: { locale: "ja-JP" } };
export const French: Story = { args: { locale: "fr-FR" } };
