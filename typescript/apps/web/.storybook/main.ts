import type { StorybookConfig } from "@storybook/nextjs-vite";

const config: StorybookConfig = {
  staticDirs: [
    { from: "../public/fonts", to: "/fonts" },
    { from: "../../../packages/ui/assets/mascots", to: "/design-assets" },
  ],
  stories: [
    "../src/**/*.stories.@(ts|tsx|mdx)",
    "../../packages/ui/src/**/*.stories.tsx",
  ],
  addons: [
    "@storybook/addon-docs",
    "@storybook/addon-a11y",
    "@storybook/addon-themes",
  ],
  framework: { name: "@storybook/nextjs-vite", options: {} },
};

export default config;
