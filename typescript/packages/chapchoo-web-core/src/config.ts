import { isIP } from "node:net";

export interface WebConfig {
  app: "general" | "admin";
  publicUrl: string;
  origin: string;
  basePath: string;
  bindHost: string;
  backendUrl: string;
  secure: boolean;
}

export interface WebEnvironment {
  publicUrl: string;
  bindHost: string;
  backendUrl: string;
}

export function isLoopback(host: string): boolean {
  const normalized = host.replace(/^\[|\]$/g, "").toLowerCase();
  if (normalized === "localhost" || normalized === "::1") {
    return true;
  }
  return isIP(normalized) === 4 && normalized.split(".")[0] === "127";
}

export function webConfig(
  app: WebConfig["app"],
  input: WebEnvironment,
): WebConfig {
  const fail = (): never => {
    throw new Error("Invalid web listener or endpoint configuration");
  };
  let publicUrl: URL;
  let backend: URL;
  try {
    publicUrl = new URL(input.publicUrl);
    backend = new URL(input.backendUrl);
  } catch {
    return fail();
  }
  for (const url of [publicUrl, backend]) {
    if (
      !["http:", "https:"].includes(url.protocol) ||
      url.username ||
      url.password ||
      url.search ||
      url.hash
    ) {
      return fail();
    }
  }
  const secure = publicUrl.protocol === "https:";
  if (
    !input.bindHost ||
    (!secure &&
      (!isLoopback(publicUrl.hostname) || !isLoopback(input.bindHost)))
  ) {
    return fail();
  }
  if (backend.pathname !== "/") {
    return fail();
  }
  const basePath = publicUrl.pathname.replace(/\/$/, "");
  if (basePath.includes("//") || /[;%\\]/.test(basePath)) {
    return fail();
  }
  return {
    app,
    publicUrl: publicUrl.toString(),
    origin: publicUrl.origin,
    basePath,
    bindHost: input.bindHost,
    backendUrl: backend.toString(),
    secure,
  };
}
