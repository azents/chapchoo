import { Stack, TextInput } from "@mantine/core";
import { useColorScheme } from "@mantine/hooks";
import { useState } from "react";
import { ColorModeSelector } from "./ColorModeSelector";
import { AppMantineProvider } from "./provider";
import type { ColorModeSelectorProps } from "./ColorModeSelector";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const meta = {
  title: "Common UI/Appearance",
  component: ColorModeSelector,
  args: {
    value: "system",
    label: "Appearance",
    labels: { system: "System", light: "Light", dark: "Dark" },
    onChange: (): void => {},
  },
} satisfies Meta<typeof ColorModeSelector>;
export default meta;
type Story = StoryObj<typeof meta>;
export const System: Story = {};
export const Light: Story = { args: { value: "light" } };
export const Dark: Story = { args: { value: "dark" } };

function InteractiveFixture(): ReactElement {
  const [value, setValue] = useState<ColorModeSelectorProps["value"]>("system");
  const [draft, setDraft] = useState("");
  const system = useColorScheme("light", { getInitialValueInEffect: false });
  return (
    <AppMantineProvider forceColorScheme={value === "system" ? system : value}>
      <Stack p="lg" maw="30rem">
        <ColorModeSelector
          value={value}
          label="Appearance"
          labels={{ system: "System", light: "Light", dark: "Dark" }}
          onChange={setValue}
        />
        <TextInput
          label="Draft"
          value={draft}
          onChange={(event) => setDraft(event.currentTarget.value)}
        />
      </Stack>
    </AppMantineProvider>
  );
}
export const Interactive: Story = {
  parameters: { ownsAppearanceProvider: true },
  render: () => <InteractiveFixture />,
};
