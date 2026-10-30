import { randomBytes } from "node:crypto";

/** Broad masking deliberately prefers losing diagnostic detail to leaking a grant. */
export function sanitize(value: string): string {
  return value
    .replace(/Aa1![A-Za-z0-9_-]{43}/g, "[password]")
    .replace(/([?&](?:token|proof)=)[^&\s)\"'<>]+/gi, "$1[proof]")
    .replace(/\bBearer\s+[^\s\"'<>]+/gi, "Bearer [credential]")
    .replace(/[A-Za-z0-9_-]{43,}={0,2}/g, "[secret]");
}

export function safeMessage(value: string): string {
  const first =
    value.split("\n").find((line) => line.trim().length > 0) ??
    "Operation failed";
  return sanitize(first)
    .replace(/\u001b\[[0-9;]*m/g, "")
    .slice(0, 500);
}

export function secret(): string {
  return randomBytes(32).toString("base64url");
}

export interface FixtureUser {
  email: string;
  password: string;
}

export function fixtureUser(label: string): FixtureUser {
  return {
    email: `${label}-${randomBytes(8).toString("hex")}@example.invalid`,
    password: `Aa1!${secret()}`,
  };
}

/** Only complete, sanitized recent output lines may cross the diagnostic boundary. */
export function diagnosticTail(output: string, truncated = false): string {
  const lines = output.split(/\r?\n/);
  if (truncated) {
    lines.shift();
  }
  if (!output.endsWith("\n")) {
    lines.pop();
  }
  return lines
    .filter(
      (line) =>
        !/^\s*(?:at\s|\d+:\s|(?:stack|backtrace)\s*[:=]|[$>]|(?:command|args|arguments|argv|env|environment)\s*[:=]|[A-Z][A-Z0-9_]*=)/i.test(
          line,
        ),
    )
    .map((line) =>
      sanitize(line)
        .replace(/\bbacktrace\s*[=:].*$/i, "[backtrace omitted]")
        .replace(/\b(?:https?|postgres(?:ql)?):\/\/[^\s\"'<>]+/gi, "[url]")
        .trim(),
    )
    .filter((line) => line.length > 0)
    .slice(-6)
    .map((line) => line.slice(0, 200))
    .join(" | ");
}
