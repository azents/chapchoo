"use client";
import { RpcProvider } from "@/shared/rpc/provider";
import { containerUser } from "../chat/components/chatContainerFixture";
import { draftFor } from "../mcpDraft";
import { mcpServersCopy } from "../mcpServersCopy";
import { McpServersPage } from "../McpServersPage";
import { McpServers } from "./McpServers";
import type { McpServersProps } from "../mcpServerTypes";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const saved = {
  id: "saved",
  displayName: "Team documents",
  endpoint: "https://docs.example.com/mcp",
  hasBearer: true,
};
const noop = (): void => {};
const args: McpServersProps = {
  copy: mcpServersCopy("en"),
  list: { type: "loaded", items: [saved], more: false, previous: false },
  mobile: false,
  editor: null,
  write: "idle",
  canTest: false,
  canSave: false,
  keepsToken: false,
  onRetry: noop,
  onNext: noop,
  onPrevious: noop,
  onOpen: noop,
  onClose: noop,
  onDraft: noop,
  onTest: noop,
  onSave: noop,
  onConfirmDelete: noop,
  onDelete: noop,
  onCheck: noop,
};
function Transport(): ReactElement {
  return (
    <RpcProvider initialIdentity={{ type: "LOADED", data: containerUser }}>
      <McpServersPage agentId="agent" />
    </RpcProvider>
  );
}

const meta = {
  title: "Computers/MCP servers",
  component: McpServers,
  args,
} satisfies Meta<typeof McpServers>;
export default meta;
type Story = StoryObj<typeof meta>;
export const List: Story = {};
export const Edit: Story = {
  args: {
    editor: {
      saved,
      draft: draftFor(saved),
      confirming: false,
      rejected: false,
      test: { type: "failed", reason: "authentication" },
    },
    canSave: true,
    canTest: true,
    keepsToken: true,
  },
};
export const SettingsTransport: Story = { render: Transport };
