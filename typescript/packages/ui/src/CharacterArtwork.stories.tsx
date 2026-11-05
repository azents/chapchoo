import { Group } from "@mantine/core";
import { CharacterArtwork } from "./CharacterArtwork";
import { mascotAssets, mascotIds } from "./designTokens";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Shared/Character artwork",
  component: CharacterArtwork,
  args: {
    assetKey: "01-retriever",
    style: { width: "10rem", height: "10rem" },
  },
} satisfies Meta<typeof CharacterArtwork>;
export default meta;
type Story = StoryObj<typeof meta>;
export const AllCharacters: Story = {
  render: (args): ReactElement => (
    <Group>
      {mascotIds.map((id) => (
        <CharacterArtwork
          {...args}
          key={id}
          assetKey={mascotAssets[id].asset}
        />
      ))}
    </Group>
  ),
};
export const InsideNavigation: Story = {
  render: (args): ReactElement => (
    <a href="/chosen-agent" aria-label="Open Agent">
      <CharacterArtwork {...args} />
    </a>
  ),
};
