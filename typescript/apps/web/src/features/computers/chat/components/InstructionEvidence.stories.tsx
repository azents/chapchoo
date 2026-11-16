import { instructionEvidenceCopy } from "../instructionEvidenceCopy";
import { InstructionEvidence } from "./InstructionEvidence";
import type { InstructionEvidenceView } from "@chapchoo/web-core/instruction-evidence";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const prompt: InstructionEvidenceView = {
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
      text: "  Preserve this exact instruction.\n한국어 원문도 그대로 유지합니다.\n",
    },
  ],
  document: "Original selected project document",
};
const meta = {
  title: "Chat/System prompt",
  component: InstructionEvidence,
  args: {
    basePath: "",
    teamHandle: "my",
    agentHandle: "companion",
    identity: "owner@example.test",
    copy: instructionEvidenceCopy("en"),
    agentsLabel: "Agents",
    replayLabel: "Replay mascot",
    appearance: {
      value: "system",
      label: "Appearance",
      labels: { system: "System", light: "Light", dark: "Dark" },
      onChange: (): void => {},
    },
    prompt,
  },
} satisfies Meta<typeof InstructionEvidence>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Applied: Story = {};
export const Prepared: Story = {
  args: { prompt: { ...prompt, appliedAt: void 0 } },
};
export const Empty: Story = { args: { prompt: null } };
export const Korean: Story = { args: { copy: instructionEvidenceCopy("ko") } };
