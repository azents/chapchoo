import { Button, Stack, Text } from "@mantine/core";
import { AgentTheme } from "./AgentTheme";
import { ResponsiveDialog } from "./ResponsiveDialog";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Shared UI/Agent theme",
  component: AgentTheme,
  args: {
    palette: "rose",
    children: (
      <Stack p="xl">
        <Button>Agent action</Button>
        <ResponsiveDialog
          mobile
          opened
          title="Agent dialog"
          closeLabel="Close"
          locked={false}
          onClose={(): void => {}}
        >
          <Text>Agent portal content</Text>
          <Button>Portal action</Button>
        </ResponsiveDialog>
      </Stack>
    ),
  },
} satisfies Meta<typeof AgentTheme>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Rose: Story = {};
export const Teal: Story = { args: { palette: "teal" } };
