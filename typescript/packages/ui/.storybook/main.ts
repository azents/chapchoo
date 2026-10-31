import type { StorybookConfig } from "@storybook/nextjs-vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.tsx"],
  staticDirs: [
    { from: "../assets/fonts", to: "/fonts" },
    { from: "../assets/illustrations", to: "/design-assets" },
  ],
  framework: { name: "@storybook/nextjs-vite", options: {} },
};
export default config;
