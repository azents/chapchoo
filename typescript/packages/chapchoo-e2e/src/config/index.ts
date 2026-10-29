import { chmodSync, lstatSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// This module defines the environment boundary itself; it is not a barrel.
export const privateOutputRoot = fileURLToPath(
  new URL("../../test-results/", import.meta.url),
);
export const runnerOutput = join(privateOutputRoot, "run");

/** The runner may recreate `run`; its enclosing private directory stays 0700. */
export function preparePrivateOutput(): string {
  mkdirSync(privateOutputRoot, { recursive: true, mode: 0o700 });
  const stat = lstatSync(privateOutputRoot);
  if (
    !stat.isDirectory() ||
    stat.isSymbolicLink() ||
    stat.uid !== process.getuid?.()
  ) {
    throw new Error("Acceptance output must be an owned private directory");
  }
  chmodSync(privateOutputRoot, 0o700);
  return runnerOutput;
}

/** App children never inherit backend, installation, setup, AWS or Node preload env. */
export function safeEnvironment(): NodeJS.ProcessEnv {
  const result: NodeJS.ProcessEnv = {
    NEXT_TELEMETRY_DISABLED: "1",
    TURBO_TELEMETRY_DISABLED: "1",
  };
  for (const name of [
    "PATH",
    "HOME",
    "LANG",
    "LC_ALL",
    "TZ",
    "SWC_NATIVE_BINDING_CACHE",
  ]) {
    const value = process.env[name];
    if (value) {
      result[name] = value;
    }
  }
  return result;
}

export function buildEnvironment(): NodeJS.ProcessEnv {
  const result = safeEnvironment();
  for (const name of ["CARGO_HOME", "RUSTUP_HOME"]) {
    const value = process.env[name];
    if (value) {
      result[name] = value;
    }
  }
  return result;
}
