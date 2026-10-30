import { defineConfig } from "@playwright/test";
import { preparePrivateOutput } from "./src/config/index.js";

export default defineConfig({
  testDir: "./src",
  testMatch: "**/*.spec.ts",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 60_000,
  expect: { timeout: 15_000 },
  forbidOnly: true,
  reporter: [["./src/reporter.ts"]],
  outputDir: preparePrivateOutput(),
  preserveOutput: "never",
  use: {
    browserName: "chromium",
    locale: "en-US",
    viewport: { width: 1280, height: 900 },
    trace: "off",
    video: "off",
    screenshot: "off",
    actionTimeout: 15_000,
    navigationTimeout: 30_000,
  },
});
