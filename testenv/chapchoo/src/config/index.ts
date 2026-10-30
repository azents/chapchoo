import { realpathSync, statSync } from "node:fs";
import { isAbsolute, join } from "node:path";

/** An explicit archive run never falls back to source builds or host Node. */
export function installedPackageRoot(): string | null {
  const value = process.env.CHAPCHOO_E2E_PACKAGE_ROOT;
  if (value === void 0) {
    return null;
  }
  if (!isAbsolute(value)) {
    throw new Error(
      "Installed package acceptance requires an absolute artifact root",
    );
  }
  const root = realpathSync(value);
  if (
    !statSync(root).isDirectory() ||
    !statSync(join(root, "chapchoo")).isFile() ||
    !statSync(join(root, "runtime/bin/node")).isFile()
  ) {
    throw new Error(
      "Installed package acceptance requires complete built artifacts",
    );
  }
  return root;
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
