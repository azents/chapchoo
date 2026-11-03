import { Button, Group, Stack, Text } from "@mantine/core";
import { useState } from "react";
import { DropdownButton } from "./DropdownButton";
import { NavigationIcon } from "./NavigationIcon";
import type { DropdownButtonProps } from "./DropdownButton";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function InteractiveDropdown(props: DropdownButtonProps): ReactElement {
  const [opened, setOpened] = useState(props.opened);
  return (
    <Stack p="lg" align="flex-start">
      <DropdownButton {...props} opened={opened} onChange={setOpened} />
      <Button variant="default" mt="12rem">
        Outside action
      </Button>
    </Stack>
  );
}
const meta = {
  title: "Controls/Dropdown button",
  component: DropdownButton,
  args: {
    accessibleLabel: "Choose workspace",
    label: (
      <Group gap="sm" wrap="nowrap">
        <Text>Workspace</Text>
        <NavigationIcon name="down" size="sm" />
      </Group>
    ),
    children: (
      <Stack gap="xs">
        <Button variant="subtle">Personal space</Button>
        <Button variant="subtle">Design team</Button>
      </Stack>
    ),
    opened: false,
    onChange: (): void => {},
  },
  render: (args): ReactElement => <InteractiveDropdown {...args} />,
} satisfies Meta<typeof DropdownButton>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Interactive: Story = {};
export const Opened: Story = { args: { opened: true } };
export const InlineOverlay: Story = { args: { withinPortal: false } };
