import { Button, Group, Stack, Text } from "@mantine/core";
import { AgentTheme } from "./AgentTheme";
import { FloatingSurface, floatingSurfaceClassName } from "./FloatingSurface";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture({ profile = false }: { profile?: boolean }): ReactElement {
  return (
    <AgentTheme palette={profile ? "teal" : null}>
      <Stack p="xl" gap="xl">
        <Text>
          Floating controls reuse the status-summary material. Content remains
          opaque.
        </Text>
        <Group>
          <FloatingSurface
            style={{ padding: "0.75rem 1rem", borderRadius: "1.75rem" }}
          >
            Status summary
          </FloatingSurface>
          <Button
            variant="subtle"
            className={floatingSurfaceClassName}
            radius="xl"
          >
            Scroll down
          </Button>
        </Group>
      </Stack>
    </AgentTheme>
  );
}
const meta: Meta<typeof Fixture> = {
  title: "FloatingSurface",
  component: Fixture,
};
export default meta;
type Story = StoryObj<typeof meta>;
export const Neutral: Story = {};
export const Profile: Story = { args: { profile: true } };
