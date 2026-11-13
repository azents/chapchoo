import { DisplayProvider } from "@/shared/preferences/DisplayProvider";
import { RpcProvider } from "@/shared/rpc/provider";
import { useChatContainer } from "../containers/useChatContainer";
import { Chat } from "./Chat";
import { containerInput, containerUser } from "./chatContainerFixture";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Container(): ReactElement {
  const props = useChatContainer(containerInput);
  return <Chat {...props} />;
}
/** Transport fixture: executable component tests intercept every API request. */
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
        <Container />
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
