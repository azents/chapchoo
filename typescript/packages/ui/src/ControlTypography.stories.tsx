import { Button, Chip, Group, Stack, rem } from "@mantine/core";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function ControlTypography(): ReactElement {
  return (
    <Stack maw={rem(420)} p="lg">
      <Button>Log in</Button>
      <Group>
        <Button variant="subtle" size="xs">
          Log out
        </Button>
        <Button variant="default">다시 로그인</Button>
        <Chip defaultChecked>Typography gyp 한글</Chip>
      </Group>
      <Button variant="outline">
        Save your typography preferences and continue editing
      </Button>
      <Button loading>Saving typography</Button>
    </Stack>
  );
}

const meta = {
  title: "Shared UI/Control typography",
  component: ControlTypography,
} satisfies Meta<typeof ControlTypography>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Labels: Story = {};
