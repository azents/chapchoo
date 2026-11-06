import { Box, rem } from "@mantine/core";
import { creationCopy } from "../creationCopy";
import { ProfilePaletteChoices } from "./ProfileChoices";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const meta = {
  title: "Computers/Profile palette choices",
  component: ProfilePaletteChoices,
  args: {
    copy: creationCopy("ko"),
    selected: "butter",
    disabled: false,
    onChange: (): void => {},
    onRandom: (): void => {},
  },
  decorators: [
    (Story) => (
      <Box maw={rem(320)} p="lg">
        <Story />
      </Box>
    ),
  ],
} satisfies Meta<typeof ProfilePaletteChoices>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Narrow: Story = {};
export const Disabled: Story = { args: { disabled: true } };
