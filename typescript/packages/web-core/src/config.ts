import { isIP } from "node:net";

export interface PublicWebConfig {
  publicUrl: string;
  origin: string;
  basePath: string;
  bindHost: string;
  secure: boolean;
}
export interface WebConfig extends PublicWebConfig {
  app: "general" | "admin";
  backendUrl: string;
}
export interface PublicWebEnvironment {
  publicUrl: string;
  bindHost: string;
}
export interface WebEnvironment extends PublicWebEnvironment {
  backendUrl: string;
}

export function isLoopback(host: string): boolean {
  const normalized = host.replace(/^\[|\]$/g, "").toLowerCase();
  if (normalized === "localhost" || normalized === "::1") {
    return true;
  }
  return isIP(normalized) === 4 && normalized.split(".")[0] === "127";
}
function invalidConfiguration(): never {
  throw new Error("Invalid web listener or endpoint configuration");
}
function endpoint(value: string): URL {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return invalidConfiguration();
  }
  if (
    !["http:", "https:"].includes(url.protocol) ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    return invalidConfiguration();
  }
  return url;
}

/** Public navigation metadata never supplies an API client destination. */
export function webDestination(value: string | undefined): string | null {
  if (!value) return null;
  const url = endpoint(value);
  if (url.protocol === "http:" && !isLoopback(url.hostname))
    invalidConfiguration();
  const path = url.pathname.replace(/\/$/, "");
  if (path.includes("//") || /[;%\\]/.test(path)) invalidConfiguration();
  return url.origin + path;
}

/** Public metadata/build consumers do not require a deployment backend target. */
export function publicWebConfig(input: PublicWebEnvironment): PublicWebConfig {
  const publicUrl = endpoint(input.publicUrl);
  const secure = publicUrl.protocol === "https:";
  if (
    !input.bindHost ||
    (!secure &&
      (!isLoopback(publicUrl.hostname) || !isLoopback(input.bindHost)))
  ) {
    return invalidConfiguration();
  }
  const basePath = publicUrl.pathname.replace(/\/$/, "");
  if (basePath.includes("//") || /[;%\\]/.test(basePath)) {
    return invalidConfiguration();
  }
  return {
    publicUrl: publicUrl.toString(),
    origin: publicUrl.origin,
    basePath,
    bindHost: input.bindHost,
    secure,
  };
}

/** Runtime backend consumers require their own explicit validated endpoint. */
export function webConfig(
  app: WebConfig["app"],
  input: WebEnvironment,
): WebConfig {
  const publicConfig = publicWebConfig(input);
  const backend = endpoint(input.backendUrl);
  if (backend.pathname !== "/") {
    return invalidConfiguration();
  }
  return { ...publicConfig, app, backendUrl: backend.toString() };
}
