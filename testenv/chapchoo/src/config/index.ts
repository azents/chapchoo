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
  for (const name of ["CARGO_HOME", "RUSTUP_HOME", "CARGO_INCREMENTAL"]) {
    const value = process.env[name];
    if (value) {
      result[name] = value;
    }
  }
  return result;
}

/** A selected native artifact never falls back to a source build. */
export function integrationBinary(): string | null {
  const value = process.env.CHAPCHOO_INTEGRATION_BINARY;
  if (value === void 0) {
    return null;
  }
  try {
    const file = statSync(value);
    if (!isAbsolute(value) || !file.isFile() || (file.mode & 0o111) === 0) {
      throw new Error("Invalid binary");
    }
    return realpathSync(value);
  } catch {
    throw new Error(
      "Native integration requires an absolute existing built executable",
    );
  }
}

export interface DockerFixtureConfig {
  socket: string;
  image: string;
  controlHost: string | null;
}

/** Opt-in authorizes only an operator-selected disposable fixture daemon. */
export function dockerFixtureConfig(): DockerFixtureConfig {
  const socket = process.env.CHAPCHOO_INTEGRATION_DOCKER_SOCKET;
  const image = process.env.CHAPCHOO_INTEGRATION_DOCKER_IMAGE;
  const controlHost =
    process.env.CHAPCHOO_INTEGRATION_DOCKER_CONTROL_HOST ?? null;
  if (
    process.env.CHAPCHOO_INTEGRATION_DOCKER_AUTHORIZED !== "disposable" ||
    !socket ||
    !isAbsolute(socket) ||
    !image ||
    image.length > 1024 ||
    /\s/.test(image) ||
    (controlHost !== null && !/^[a-zA-Z0-9.-]+$/.test(controlHost))
  ) {
    throw new Error(
      "Real Docker acceptance requires an authorized disposable socket, current built image and reachable Control host",
    );
  }
  try {
    if (!statSync(socket).isSocket()) {
      throw new Error("Not a Unix socket");
    }
  } catch {
    throw new Error(
      "Real Docker acceptance requires an authorized disposable socket; selected socket is unavailable",
    );
  }
  // docker-modem reads environment defaults before explicit options. Prevent
  // those defaults from supplying another host, TLS bundle or path prefix.
  for (const name of [
    "DOCKER_HOST",
    "DOCKER_CERT_PATH",
    "DOCKER_TLS_VERIFY",
    "DOCKER_TLS",
    "DOCKER_PATH_PREFIX",
  ]) {
    if (process.env[name]) {
      throw new Error(
        "Docker acceptance SDK requires a clean Docker transport environment; use the explicit fixture inputs",
      );
    }
  }
  return { socket, image, controlHost };
}
