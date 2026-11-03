import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  installedPackageRoot,
  safeEnvironment,
  dockerFixtureConfig,
} from "./index.ts";

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
    "CHAPCHOO_E2E_DOCKER_AUTHORIZED",
    "CHAPCHOO_E2E_DOCKER_SOCKET",
    "CHAPCHOO_E2E_DOCKER_IMAGE",
  ];
  const saved = new Map(keys.map((key) => [key, process.env[key]]));
  const root = mkdtempSync(join(tmpdir(), "chapchoo-docker-prerequisite-"));
  try {
    for (const key of keys) {
      delete process.env[key];
    }
    assert.throws(dockerFixtureConfig, /authorized disposable socket/);
    process.env.CHAPCHOO_E2E_DOCKER_AUTHORIZED = "disposable";
    process.env.CHAPCHOO_E2E_DOCKER_IMAGE = "current-built:fixture";
    process.env.CHAPCHOO_E2E_DOCKER_SOCKET = "relative.sock";
    assert.throws(dockerFixtureConfig, /authorized disposable socket/);
    const regular = join(root, "not-a-socket");
    writeFileSync(regular, "fixture");
    process.env.CHAPCHOO_E2E_DOCKER_SOCKET = regular;
    assert.throws(dockerFixtureConfig, /authorized disposable socket/);
    process.env.CHAPCHOO_E2E_DOCKER_SOCKET = join(
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
