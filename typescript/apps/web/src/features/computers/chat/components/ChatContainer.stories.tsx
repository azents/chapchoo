import { DisplayProvider } from "@/shared/preferences/DisplayProvider";
import { RpcProvider } from "@/shared/rpc/provider";
import { useChatContainer } from "../containers/useChatContainer";
import { Chat } from "./Chat";
import {
  containerFailure,
  containerInput,
  containerUser,
  deferredContainerInput,
} from "./chatContainerFixture";
import type { DeferredComputerState } from "./chatContainerFixture";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Container({
  queued,
  state,
}: {
  queued: boolean;
  state?: DeferredComputerState;
}): ReactElement {
  const props = useChatContainer(
    queued
      ? containerFailure
      : state
        ? deferredContainerInput(state)
        : containerInput,
  );
  return <Chat {...props} />;
}
/** Transport fixture: executable component tests intercept every API request. */
function Fixture({
  queued = false,
  state,
}: {
  queued?: boolean;
  state?: DeferredComputerState;
}): ReactElement {
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
        <Container queued={queued} state={state} />
      </DisplayProvider>
    </RpcProvider>
  );
}
const meta = {
  title: "Chat/Container transport",
  component: Fixture,
  parameters: { layout: "fullscreen" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const EnqueueFailure: Story = {};
export const RemovalRace: Story = { args: { queued: true } };
export const Stopped: Story = { args: { state: "stopped" } };
export const Starting: Story = { args: { state: "starting" } };
export const Restoring: Story = { args: { state: "restoring" } };
