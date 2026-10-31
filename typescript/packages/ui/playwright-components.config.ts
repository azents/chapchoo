import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./src",
  testMatch: "**/*.component.spec.ts",
  workers: 1,
  retries: 0,
  forbidOnly: true,
  use: {
    baseURL: "http://127.0.0.1:6018",
    browserName: "chromium",
    viewport: { width: 1280, height: 900 },
    locale: "en-US",
    colorScheme: "light",
    trace: "off",
    video: "off",
    screenshot: "off",
  },
  webServer: {
    command:
      "python3 -m http.server 6018 --bind 127.0.0.1 --directory storybook-static",
    url: "http://127.0.0.1:6018",
    reuseExistingServer: false,
  },
});
