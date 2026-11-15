import { instructionEvidenceCopy } from "../instructionEvidenceCopy";
import { InstructionEvidence } from "./InstructionEvidence";
import type { InstructionEvidenceView } from "@chapchoo/web-core/instruction-evidence";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const evidence: InstructionEvidenceView = {
  agentId: "agent",
  conversationId: "conversation",
  operationId: "operation",
  executionId: "execution",
  createdAt: 1000,
  appliedAt: 1001,
  sources: [
    {
      role: "product",
      text: "Publish intentional conversation messages through the supplied tool.",
    },
    {
      role: "owner",
      text: "  Preserve this exact owner source.\n한국어 원문도 그대로 유지합니다.\n",
    },
  ],
  document: "Original selected project document",
};
const meta = {
  title: "Chat/Prompt sources",
  component: InstructionEvidence,
  args: {
    opened: true,
    copy: instructionEvidenceCopy("en"),
    state: {
      type: "ready",
      selected: "operation",
      options: [{ value: "operation", label: "Execution · 2026-11-16" }],
      more: { available: false, read: "ready" },
      evidence: { type: "ready", value: evidence },
    },
    onToggle: (): void => {},
    onSelect: (): void => {},
    onRead: (): void => {},
    onMore: (): void => {},
  },
} satisfies Meta<typeof InstructionEvidence>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Supplied: Story = {};
export const Prepared: Story = {
  args: {
    state: {
      type: "ready",
      selected: "operation",
      options: [{ value: "operation", label: "Prepared execution" }],
      more: { available: false, read: "ready" },
      evidence: { type: "ready", value: { ...evidence, appliedAt: void 0 } },
    },
  },
};
export const Loading: Story = { args: { state: { type: "loading" } } };
export const ReadFailure: Story = { args: { state: { type: "error" } } };
export const Empty: Story = { args: { state: { type: "empty" } } };
export const SourceReadFailure: Story = {
  args: {
    state: {
      type: "ready",
      selected: "operation",
      options: [{ value: "operation", label: "Execution" }],
      more: { available: false, read: "ready" },
      evidence: { type: "error" },
    },
  },
};
export const OlderLoading: Story = {
  args: {
    state: {
      ...meta.args.state,
      more: { available: true, read: "loading" },
    },
  },
};
export const OlderReadFailure: Story = {
  args: {
    state: {
      ...meta.args.state,
      more: { available: true, read: "error" },
    },
  },
};
