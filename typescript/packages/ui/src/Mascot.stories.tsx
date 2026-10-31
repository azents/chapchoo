import { Group, Stack, Text } from "@mantine/core";
import { Mascot } from "./Mascot";
import { mascotAssets } from "./designTokens";
import type { MascotId } from "./designTokens";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = { title: "Shared/Mascot", component: Mascot } satisfies Meta<
  typeof Mascot
>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Retriever: Story = { args: { size: 72 } };
export const Family: Story = {
  render: (): ReactElement => {
    const characters: MascotId[] = [
      "retriever",
      "bichon",
      "corgi",
      "dachshund",
      "husky",
      "collie",
    ];
    return (
      <Group align="start">
        {characters.map((character) => (
          <Stack key={character} align="center" gap="xs">
            <Mascot character={character} size={72} />
            <Text size="sm">{mascotAssets[character].name}</Text>
          </Stack>
        ))}
      </Group>
    );
  },
};
