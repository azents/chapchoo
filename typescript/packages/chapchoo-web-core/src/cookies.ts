import type { WebConfig } from "./config.js";

export interface CookiePolicy {
  name: string;
  path: string;
  secure: boolean;
  sameSite: "Lax" | "None";
}

export function cookiePolicy(config: WebConfig): CookiePolicy {
  if (config.app === "general") {
    return {
      name: config.secure
        ? "__Host-Chapchoo-General"
        : "chapchoo-general-session",
      path: "/",
      secure: config.secure,
      sameSite: config.secure ? "None" : "Lax",
    };
  }
  const path = config.basePath || "/";
  return {
    name: config.secure
      ? path === "/"
        ? "__Host-Chapchoo-Admin"
        : "__Secure-Chapchoo-Admin"
      : "chapchoo-admin-session",
    path,
    secure: config.secure,
    sameSite: "Lax",
  };
}

export function readSessionCookie(
  header: string | null,
  policy: CookiePolicy,
): string | null {
  const values = (header ?? "")
    .split(";")
    .map((value) => value.trim())
    .filter((value) => value.startsWith(`${policy.name}=`));
  if (values.length !== 1) {
    return null;
  }
  const value = values[0];
  if (value == null) {
    return null;
  }
  const token = value.slice(policy.name.length + 1);
  return /^[A-Za-z0-9_-]{43}$/.test(token) ? token : null;
}

function attributes(policy: CookiePolicy): string {
  return `Path=${policy.path}; HttpOnly; SameSite=${policy.sameSite}${policy.secure ? "; Secure" : ""}`;
}

export function sessionCookie(
  policy: CookiePolicy,
  token: string,
  serverExpiresAt: number,
  now: number,
): string {
  if (
    !/^[A-Za-z0-9_-]{43}$/.test(token) ||
    !Number.isFinite(serverExpiresAt) ||
    serverExpiresAt <= now
  ) {
    throw new Error("Invalid authenticated cookie result");
  }
  const maxAge = Math.max(
    0,
    Math.floor(Math.min(30 * 24 * 60 * 60, (serverExpiresAt - now) / 1000)),
  );
  const expires = new Date(now + maxAge * 1000).toUTCString();
  return `${policy.name}=${token}; ${attributes(policy)}; Max-Age=${maxAge}; Expires=${expires}`;
}

export function clearSessionCookie(policy: CookiePolicy): string {
  return `${policy.name}=; ${attributes(policy)}; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT`;
}
