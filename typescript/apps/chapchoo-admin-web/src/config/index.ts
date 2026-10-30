/* eslint-disable check-file/no-index -- This defining module is the required environment boundary, not a barrel. */
import { webConfig } from "@chapchoo/web-core/config";
import type { WebConfig } from "@chapchoo/web-core/config";

// Next configuration executes in plain Node before its server-only aliases exist.
// All runtime consumers additionally carry the framework server-only marker.
if (typeof window !== "undefined") {
  throw new Error("Web server configuration is available only in Node.js");
}

export function getBuildConfig(): { basePath: string } {
  const config = webConfig("admin", {
    publicUrl: process.env.CHAPCHOO_PUBLIC_URL ?? "http://localhost:3004",
    bindHost: "127.0.0.1",
    backendUrl: "http://127.0.0.1:1",
  });
  return { basePath: config.basePath };
}

export function getConfig(): WebConfig {
  const config = webConfig("admin", {
    publicUrl: process.env.CHAPCHOO_PUBLIC_URL ?? "http://localhost:3004",
    bindHost: process.env.CHAPCHOO_WEB_HOST ?? "127.0.0.1",
    backendUrl: process.env.CHAPCHOO_BACKEND_URL ?? "http://127.0.0.1:50052",
  });
  // nextConfig.env inlines this non-secret build constant in Node output.
  if (process.env.CHAPCHOO_COMPILED_BASE_PATH !== config.basePath) {
    throw new Error("Configured public base path does not match the web build");
  }
  return config;
}

/** Safe server-to-provider values; private endpoints stay on the Node side. */
export function getPublicConfig(): { publicBasePath: string } {
  return { publicBasePath: getConfig().basePath };
}
