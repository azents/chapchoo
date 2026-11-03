import { Group, Stack, Text } from "@mantine/core";
import { useState } from "react";
import { InteractiveSurface } from "./InteractiveSurface";
import { NavigationIcon } from "./NavigationIcon";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Chrome/Interactive surface",
  component: InteractiveSurface,
  parameters: { layout: "padded" },
  args: { children: "Open settings" },
} satisfies Meta<typeof InteractiveSurface>;
export default meta;
type Story = StoryObj<typeof meta>;
function SurfaceExamples(): ReactElement {
  const [count, setCount] = useState(0);
  return (
    <Stack maw="24rem">
      <InteractiveSurface href="#profile">
        <Group justify="space-between">
          <Text>Open profile</Text>
          <NavigationIcon name="forward" size="sm" />
        </Group>
      </InteractiveSurface>
      <InteractiveSurface onClick={() => setCount((value) => value + 1)}>
        Open settings
      </InteractiveSurface>
      <InteractiveSurface disabled>Unavailable action</InteractiveSurface>
      <Text role="status">Actions: {count}</Text>
    </Stack>
  );
}
export const States: Story = {
  render: (): ReactElement => <SurfaceExamples />,
};
