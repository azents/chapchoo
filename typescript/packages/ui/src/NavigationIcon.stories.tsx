import { Button, Group, Stack, Text } from "@mantine/core";
import { NavigationIcon } from "./NavigationIcon";
import type { NavigationIconProps } from "./NavigationIcon";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Chrome/Navigation icons",
  component: NavigationIcon,
  args: { name: "down", size: "md" },
} satisfies Meta<typeof NavigationIcon>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Geometry: Story = {
  render: (): ReactElement => {
    const names: NavigationIconProps["name"][] = [
      "personal",
      "team",
      "admin",
      "back",
      "down",
      "add",
      "connections",
      "cloud",
    ];
    return (
      <Stack>
        {names.map((name) => (
          <Group key={name} gap="sm" wrap="nowrap">
            <NavigationIcon name={name} />
            <Text>{name}</Text>
          </Group>
        ))}
      </Stack>
    );
  },
};
export const Controls: Story = {
  render: (): ReactElement => (
    <Group>
      <Button variant="subtle" leftSection={<NavigationIcon name="back" />}>
        뒤로
      </Button>
      <Button
        variant="subtle"
        rightSection={<NavigationIcon name="down" size="sm" />}
      >
        작업 공간
      </Button>
    </Group>
  ),
};
