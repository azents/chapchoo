import { Button, Stack, Text, TextInput } from "@mantine/core";
import { ResponsiveDialog } from "./ResponsiveDialog";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Shared UI/Responsive dialog",
  component: ResponsiveDialog,
  args: {
    opened: true,
    mobile: true,
    locked: false,
    title: "Dialog heading",
    closeLabel: "Close",
    onClose: (): void => {},
    children: (
      <Stack>
        <TextInput label="Alias" />
        <Button>Save</Button>
      </Stack>
    ),
  },
} satisfies Meta<typeof ResponsiveDialog>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Short: Story = {};
export const Long: Story = {
  args: {
    children: (
      <Stack>
        {Array.from({ length: 30 }, (_, i) => (
          <Text key={i}>Long content row {i + 1}</Text>
        ))}
        <Button>Save</Button>
      </Stack>
    ),
  },
};
export const Locked: Story = { args: { locked: true } };
export const Desktop: Story = { args: { mobile: false } };
