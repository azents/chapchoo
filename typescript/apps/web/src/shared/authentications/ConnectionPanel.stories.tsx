import { useMediaQuery } from "@mantine/hooks";
import { useState } from "react";
import { fixtureConnection, fixturePanel } from "./connectionFixture";
import { ConnectionPanel } from "./ConnectionPanel";
import type { ConnectionDialog, ConnectionPanelProps } from "./types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";
function Interactive(props: ConnectionPanelProps): ReactElement {
  const mobile = useMediaQuery("(max-width: 48em)", false);
  const [dialog, setDialog] = useState<ConnectionDialog>(props.dialog);
  const [name, setName] = useState(props.name);
  const [key, setKey] = useState(props.apiKey);
  const [after, setAfter] = useState(props.after);
  return (
    <ConnectionPanel
      {...props}
      mobile={mobile}
      after={after}
      onPage={setAfter}
      dialog={dialog}
      name={name}
      apiKey={key}
      onDialog={setDialog}
      onName={setName}
      onKey={setKey}
      onClose={() => {
        setDialog({ type: "closed" });
        setKey("");
      }}
      onManage={(target) => setDialog({ type: "manage", target })}
    />
  );
}
const meta = {
  title: "Authentication/Connections",
  component: ConnectionPanel,
  args: fixturePanel(),
  render: Interactive,
} satisfies Meta<typeof ConnectionPanel>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Populated: Story = {};
export const WithoutAlias: Story = {
  args: { items: [{ ...fixtureConnection, name: "" }] },
};
export const OptionalAlias: Story = {
  args: {
    dialog: { type: "key", target: null },
    name: "",
    apiKey: "synthetic-key",
  },
};
export const LaterPage: Story = { args: { after: "cursor-page-3" } };
export const Empty: Story = { args: { items: [] } };
export const Unavailable: Story = { args: { unavailable: true } };
export const ReadOnlyTeam: Story = {
  args: { scope: { type: "team", teamId: "team" }, canManage: false },
};
export const Picker: Story = {
  args: {
    selectable: true,
    selectedId: fixtureConnection.id,
    scopes: [{ type: "personal" }, { type: "team", teamId: "team" }],
  },
};
export const ChooseMethod: Story = { args: { dialog: { type: "connect" } } };
export const KeyEntry: Story = {
  args: { dialog: { type: "key", target: null }, name: "개발용 API" },
};
export const ReplaceKey: Story = {
  args: {
    dialog: {
      type: "key",
      target: {
        ...fixtureConnection,
        kind: "openaiApiKey",
        accountId: null,
        email: null,
        planType: null,
        expiresAt: null,
      },
    },
  },
};
export const DeviceCode: Story = {
  args: {
    dialog: {
      type: "device",
      progress: {
        attemptId: "attempt",
        expiresAt: 1794000000000,
        state: {
          type: "pending",
          userCode: "ABCD-EFGH",
          verificationUri: "https://auth.openai.com/codex/device",
          nextCheckAt: 1793923200000,
        },
        providerFailure: null,
      },
    },
  },
};
export const UncertainRequest: Story = {
  args: {
    dialog: {
      type: "uncertain",
      requestId: "request",
      operation: "record",
      attemptId: null,
    },
  },
};
export const AcceptedUnavailable: Story = {
  args: { dialog: { type: "recordUnavailable", authenticationId: "removed" } },
};
export const InUse: Story = {
  args: {
    dialog: { type: "delete", target: fixtureConnection },
    error: fixturePanel().copy.inUse,
  },
};
