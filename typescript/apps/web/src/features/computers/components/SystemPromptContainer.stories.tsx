"use client";
import { RpcProvider } from "@/shared/rpc/provider";
import { containerUser } from "../chat/components/chatContainerFixture";
import { SystemPromptPage } from "../SystemPromptPage";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture(): ReactElement {
  return (
    <RpcProvider initialIdentity={{ type: "LOADED", data: containerUser }}>
      <SystemPromptPage agentId="agent" unavailable={false} />
    </RpcProvider>
  );
}
const meta = {
  title: "Computers/System prompt transport",
  component: Fixture,
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Editor: Story = {};
