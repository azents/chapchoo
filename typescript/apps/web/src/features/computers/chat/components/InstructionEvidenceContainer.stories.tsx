import { DisplayProvider } from "@/shared/preferences/DisplayProvider";
import { RpcProvider } from "@/shared/rpc/provider";
import { InstructionEvidencePage } from "../InstructionEvidencePage";
import { containerUser } from "./chatContainerFixture";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

/** Transport fixture: component tests intercept every API request. */
function Fixture(): ReactElement {
  const value = { language: "en", region: "en-US", timeZone: "UTC" };
  const identity = { type: "LOADED" as const, data: containerUser };
  return (
    <RpcProvider initialIdentity={identity}>
      <DisplayProvider
        basePath=""
        initial={{
          identity,
          value,
          defaults: value,
          explicit: {},
          instant: 1,
          zones: ["UTC"],
        }}
      >
        <InstructionEvidencePage
          agentId="agent"
          conversationId="conversation"
        />
      </DisplayProvider>
    </RpcProvider>
  );
}
const meta = {
  title: "Chat/Prompt sources transport",
  component: Fixture,
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const History: Story = {};
