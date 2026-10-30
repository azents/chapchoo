import { Button, Stack, TextInput } from "@mantine/core";
import { ContentSection } from "./ContentSection";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Shared UI/ContentSection",
  component: ContentSection,
  args: {
    title: "Team settings",
    description: "Update the team's name and handle.",
    children: (
      <Stack>
        <TextInput label="Team name" defaultValue="Design team" />
        <Button w="fit-content">Save changes</Button>
      </Stack>
    ),
  },
} satisfies Meta<typeof ContentSection>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Form: Story = {};
export const LongContent: Story = {
  args: {
    title: "A shared workspace with a long localized team name",
    description:
      "A longer description wraps without hiding the section's actions. This is static presentation data.",
  },
};
