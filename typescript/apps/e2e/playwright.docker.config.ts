import { defineConfig } from "@playwright/test";
import base from "./playwright.config.js";

// Real native prerequisites belong to an explicit executable acceptance lane.
export default defineConfig(base, { testMatch: "**/*.docker.ts" });
