import { Drawer, Stack, Text, TextInput } from "@mantine/core";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Shared UI/Drawer",
  component: Drawer,
  args: {
    opened: true,
    onClose: (): void => {},
    position: "right",
    size: "md",
    padding: "xl",
    title: "Panel heading",
    children: (
      <Stack gap="xl">
        <Text data-drawer-opening c="dimmed" size="sm">
          Body opening
        </Text>
        <TextInput label="Example field" defaultValue="Retained content" />
      </Stack>
    ),
  },
} satisfies Meta<typeof Drawer>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Content: Story = {};
export const WrappedHeading: Story = {
  args: { title: "A longer panel heading that wraps on narrow screens" },
};
