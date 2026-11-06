import { Stack, Text } from "@mantine/core";
import { useState } from "react";
import { RpcProvider } from "@/shared/rpc/provider";
import { fixtureCatalogue } from "./connectionFixture";
import { ConnectionPanel } from "./ConnectionPanel";
import { useConnectionPanel } from "./useConnectionPanel";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";
function Consumer(): ReactElement {
  const [selected, setSelected] = useState("");
  const props = useConnectionPanel({
    snapshots: [{ ...fixtureCatalogue, page: { items: [], nextAfter: null } }],
    onSelect: (value) => setSelected(value.name),
  });
  return (
    <Stack p="lg">
      <Text role="status">Selected: {selected}</Text>
      <ConnectionPanel {...props} />
    </Stack>
  );
}
function ManagementConsumer(): ReactElement {
  const [closed, setClosed] = useState(0);
  const props = useConnectionPanel({
    snapshots: [fixtureCatalogue],
    onCloseManage: () => setClosed((value) => value + 1),
  });
  return (
    <Stack p="lg">
      <Text role="status">Closed: {closed}</Text>
      <ConnectionPanel {...props} />
    </Stack>
  );
}
function Flow({ management = false }: { management?: boolean }): ReactElement {
  return (
    <RpcProvider
      initialIdentity={{
        type: "LOADED",
        data: {
          id: "fixture-user",
          email: "fixture@example.com",
          isAdmin: false,
          createdAt: { seconds: 0n, nanos: 0 },
          display: { language: "en", timeZone: "UTC", region: "en-US" },
        },
      }}
    >
      {management ? <ManagementConsumer /> : <Consumer />}
    </RpcProvider>
  );
}
const meta = { title: "Authentication/Flow", component: Flow } satisfies Meta<
  typeof Flow
>;
export default meta;
type Story = StoryObj<typeof meta>;
export const InPlace: Story = {};
export const Management: Story = { args: { management: true } };
