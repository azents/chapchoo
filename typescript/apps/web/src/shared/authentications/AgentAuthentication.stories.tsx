import { useState } from "react";
import { AgentAuthentication } from "./AgentAuthentication";
import { fixtureAgentAuthentication } from "./connectionFixture";
import type { AgentAuthenticationProps } from "./useAgentAuthentication";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";
function Interactive(props: AgentAuthenticationProps): ReactElement {
  const [opened, setOpened] = useState(props.opened);
  return (
    <AgentAuthentication
      {...props}
      opened={opened}
      onOpen={() => setOpened(true)}
      onClose={() => setOpened(false)}
    />
  );
}
const meta = {
  title: "Authentication/Agent selection",
  component: AgentAuthentication,
  args: fixtureAgentAuthentication(),
  render: Interactive,
} satisfies Meta<typeof AgentAuthentication>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Saved: Story = {};
export const Unselected: Story = {
  args: {
    agent: { ...fixtureAgentAuthentication().agent, authentication: null },
  },
};
export const Picker: Story = { args: { opened: true } };
export const Uncertain: Story = {
  args: {
    opened: true,
    task: {
      type: "uncertain",
      requestId: "selection",
      authenticationId: "credential",
    },
  },
};
export const Withdrawn: Story = { args: { authorized: false } };
