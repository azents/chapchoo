import type { StorybookConfig } from "@storybook/react-vite";

const config: StorybookConfig = {
  staticDirs: [
    { from: "../../ui/assets/mascots", to: "/mascots" },
    { from: "../../ui/assets/mascots", to: "/design-assets" },
    { from: "../../ui/assets/fonts", to: "/fonts" },
  ],
  stories: ["../src/**/*.stories.tsx"],
  addons: ["@storybook/addon-docs", "@storybook/addon-a11y"],
  framework: { name: "@storybook/react-vite", options: {} },
};

export default config;
