import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./test",
  testMatch: "**/*.browser.spec.mjs",
  workers: 1,
  retries: 0,
  forbidOnly: true,
  use: {
    browserName: "chromium",
    trace: "off",
    video: "off",
    screenshot: "off",
  },
});
