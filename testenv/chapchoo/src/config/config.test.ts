import assert from "node:assert/strict";
import {
  chmodSync,
  mkdtempSync,
  mkdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  assemblyArtifacts,
  installedPackageRoot,
  safeEnvironment,
  dockerFixtureConfig,
  integrationBinary,
} from "./index.ts";

void test("both browser modes reuse the package; native-only mode ignores it", () => {
  const keys = ["CHAPCHOO_E2E_PACKAGE_ROOT", "CHAPCHOO_INTEGRATION_BINARY"];
  const saved = new Map(keys.map((key) => [key, process.env[key]]));
  const root = mkdtempSync(join(tmpdir(), "chapchoo-assembly-artifacts-"));
  try {
    const packaged = join(root, "chapchoo");
    const standalone = join(root, "standalone");
    mkdirSync(join(root, "runtime/bin"), { recursive: true });
    for (const binary of [
      packaged,
      standalone,
      join(root, "runtime/bin/node"),
    ]) {
      writeFileSync(binary, "", { mode: 0o755 });
    }
    process.env.CHAPCHOO_E2E_PACKAGE_ROOT = root;
    delete process.env.CHAPCHOO_INTEGRATION_BINARY;
    const packageSelection = {
      packageRoot: installedPackageRoot(),
      nativeBinary: null,
    };
    assert.deepEqual(assemblyArtifacts(false, true), packageSelection);
    assert.deepEqual(assemblyArtifacts(true, true), packageSelection);
    process.env.CHAPCHOO_INTEGRATION_BINARY = packaged;
    assert.deepEqual(assemblyArtifacts(true, true), packageSelection);
    process.env.CHAPCHOO_INTEGRATION_BINARY = standalone;
    assert.throws(() => assemblyArtifacts(true, true), /from the same package/);
    const nativeSelection = {
      packageRoot: null,
      nativeBinary: integrationBinary(),
    };
    process.env.CHAPCHOO_E2E_PACKAGE_ROOT = "invalid-archive";
    assert.deepEqual(assemblyArtifacts(true, false), nativeSelection);
    assert.throws(
      () => assemblyArtifacts(true, true),
      /absolute artifact root/,
    );
    delete process.env.CHAPCHOO_E2E_PACKAGE_ROOT;
    assert.deepEqual(assemblyArtifacts(true, true), nativeSelection);
  } finally {
    rmSync(root, { recursive: true, force: true });
    for (const [key, value] of saved) {
      if (value === void 0) delete process.env[key];
      else process.env[key] = value;
    }
  }
});

void test("child environments omit backend, mail and Node preload authority", () => {
  const keys = [
    "CHAPCHOO_SECRET_KEY",
    "CHAPCHOO_POSTGRES_URL",
    "CHAPCHOO_SETUP_PROOF",
    "CHAPCHOO_SMTP_PASSWORD",
    "NODE_OPTIONS",
    "LD_PRELOAD",
  ];
  const saved = new Map(keys.map((key) => [key, process.env[key]]));
  try {
    for (const key of keys) {
      process.env[key] = "private-fixture-canary";
    }
    const child = safeEnvironment();
    for (const key of keys) {
      assert.equal(key in child, false);
    }
    assert.equal(child.NEXT_TELEMETRY_DISABLED, "1");
  } finally {
    for (const [key, value] of saved) {
      if (value === void 0) {
        delete process.env[key];
      } else {
        process.env[key] = value;
      }
    }
  }
});

void test("explicit package prerequisites never fall back to source execution", () => {
  const saved = process.env.CHAPCHOO_E2E_PACKAGE_ROOT;
  const root = mkdtempSync(join(tmpdir(), "chapchoo-package-prerequisite-"));
  try {
    delete process.env.CHAPCHOO_E2E_PACKAGE_ROOT;
    assert.equal(installedPackageRoot(), null);
    process.env.CHAPCHOO_E2E_PACKAGE_ROOT = "relative-package";
    assert.throws(installedPackageRoot, /absolute artifact root/);
    process.env.CHAPCHOO_E2E_PACKAGE_ROOT = root;
    assert.throws(installedPackageRoot);
    mkdirSync(join(root, "runtime/bin"), { recursive: true });
    writeFileSync(join(root, "chapchoo"), "fixture");
    writeFileSync(join(root, "runtime/bin/node"), "fixture");
    assert.equal(installedPackageRoot(), root);
  } finally {
    rmSync(root, { recursive: true, force: true });
    if (saved === void 0) {
      delete process.env.CHAPCHOO_E2E_PACKAGE_ROOT;
    } else {
      process.env.CHAPCHOO_E2E_PACKAGE_ROOT = saved;
    }
  }
});

void test("real Docker prerequisites cannot implicitly authorize a daemon or accept a regular file", () => {
  const keys = [
    "CHAPCHOO_INTEGRATION_DOCKER_AUTHORIZED",
    "CHAPCHOO_INTEGRATION_DOCKER_SOCKET",
    "CHAPCHOO_INTEGRATION_DOCKER_IMAGE",
  ];
  const saved = new Map(keys.map((key) => [key, process.env[key]]));
  const root = mkdtempSync(join(tmpdir(), "chapchoo-docker-prerequisite-"));
  try {
    for (const key of keys) {
      delete process.env[key];
    }
    assert.throws(dockerFixtureConfig, /authorized disposable socket/);
    process.env.CHAPCHOO_INTEGRATION_DOCKER_AUTHORIZED = "disposable";
    process.env.CHAPCHOO_INTEGRATION_DOCKER_IMAGE = "current-built:fixture";
    process.env.CHAPCHOO_INTEGRATION_DOCKER_SOCKET = "relative.sock";
    assert.throws(dockerFixtureConfig, /authorized disposable socket/);
    const regular = join(root, "not-a-socket");
    writeFileSync(regular, "fixture");
    process.env.CHAPCHOO_INTEGRATION_DOCKER_SOCKET = regular;
    assert.throws(dockerFixtureConfig, /authorized disposable socket/);
    process.env.CHAPCHOO_INTEGRATION_DOCKER_SOCKET = join(
      root,
      "unavailable-private-endpoint",
    );
    assert.throws(
      dockerFixtureConfig,
      (error: unknown) =>
        error instanceof Error &&
        error.message.includes("selected socket is unavailable") &&
        !error.message.includes(root),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
    for (const [key, value] of saved) {
      if (value === void 0) {
        delete process.env[key];
      } else {
        process.env[key] = value;
      }
    }
  }
});

void test("native integration artifact selection never borrows a web package or replaces a bad explicit binary", () => {
  const saved = process.env.CHAPCHOO_INTEGRATION_BINARY;
  const root = mkdtempSync(join(tmpdir(), "chapchoo-native-binary-"));
  try {
    delete process.env.CHAPCHOO_INTEGRATION_BINARY;
    assert.equal(integrationBinary(), null);
    process.env.CHAPCHOO_INTEGRATION_BINARY = "relative-binary";
    assert.throws(integrationBinary, /absolute existing built executable/);
    const binary = join(root, "chapchoo");
    writeFileSync(binary, "synthetic executable fixture");
    process.env.CHAPCHOO_INTEGRATION_BINARY = binary;
    assert.throws(integrationBinary, /absolute existing built executable/);
    chmodSync(binary, 0o755);
    assert.equal(integrationBinary(), binary);
  } finally {
    rmSync(root, { recursive: true, force: true });
    if (saved === void 0) {
      delete process.env.CHAPCHOO_INTEGRATION_BINARY;
    } else {
      process.env.CHAPCHOO_INTEGRATION_BINARY = saved;
    }
  }
});
