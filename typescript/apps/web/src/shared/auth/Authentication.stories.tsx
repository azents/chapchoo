import { useAuth } from "@chapchoo/web-core/auth-context";
import { Button, Group, Stack, Text } from "@mantine/core";
import { RpcProvider } from "@/shared/rpc/provider";
import type { InitialIdentity } from "@chapchoo/web-core/auth-context";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactNode } from "react";

function Consumer({ label }: { label: string }): ReactNode {
  const auth = useAuth();
  return (
    <Stack>
      <Text role="status">
        {label}:{" "}
        {auth.state.type === "AUTHENTICATED"
          ? auth.state.user.email
          : auth.state.type}
      </Text>
      <Group>
        <Button
          onClick={() => {
            void auth.recheck().catch(() => {});
          }}
          disabled={auth.checking}
        >
          Recheck {label}
        </Button>
        <Button onClick={auth.logout}>Log out {label}</Button>
        <Button onClick={auth.clearIdentity}>Clear {label}</Button>
      </Group>
    </Stack>
  );
}
function Authentication({ initial }: { initial: InitialIdentity }): ReactNode {
  return (
    <RpcProvider initialIdentity={initial} basePath="/auth-probe">
      <Consumer label="header" />
      <Consumer label="content" />
    </RpcProvider>
  );
}
const meta = {
  title: "Common UI/Authentication",
  component: Authentication,
} satisfies Meta<typeof Authentication>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Authenticated: Story = {
  args: {
    initial: {
      type: "LOADED",
      data: {
        id: "fixture-user",
        email: "owner@example.com",
        isAdmin: false,
        createdAt: { seconds: 0n, nanos: 0 },
        display: { language: "en", timeZone: "UTC", region: "en-US" },
      },
    },
  },
};
export const SignedOut: Story = {
  args: { initial: { type: "LOADED", data: null } },
};
export const Unavailable: Story = {
  args: { initial: { type: "ERROR", reason: null } },
};
