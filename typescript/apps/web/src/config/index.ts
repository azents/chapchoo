/* eslint-disable check-file/no-index -- This defining module is the required environment boundary, not a barrel. */
import { webConfig } from "@chapchoo/web-core/config";
import type { WebConfig } from "@chapchoo/web-core/config";

// Next configuration executes in plain Node before its server-only aliases exist.
// All runtime consumers additionally carry the framework server-only marker.
if (typeof window !== "undefined") {
  throw new Error("Web server configuration is available only in Node.js");
}

export function getConfig(): WebConfig {
  const config = webConfig("general", {
    publicUrl: process.env.CHAPCHOO_PUBLIC_URL ?? "http://localhost:3003",
    bindHost: process.env.CHAPCHOO_WEB_HOST ?? "127.0.0.1",
    backendUrl: process.env.CHAPCHOO_BACKEND_URL ?? "http://127.0.0.1:50051",
  });
  return config;
}

export function getListener(): { host: string; port: number } {
  const config = getConfig();
  const port = Number(process.env.PORT ?? "3003");
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error("Invalid web listener port");
  }
  return { host: config.bindHost, port };
}

/** Safe server-to-provider values; private endpoints stay on the Node side. */
export function getPublicConfig(): { publicBasePath: string } {
  return { publicBasePath: getConfig().basePath };
}

/** Development Next owns basePath; the production custom mount owns it itself. */
export function getLoginRedirect(): string {
  return process.env.NODE_ENV === "development"
    ? "/login"
    : `${getConfig().basePath}/login`;
}
