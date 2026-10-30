import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import test from "node:test";

void test("public paths are read from runtime configuration without compiled constants", () => {
  const source = new URL("./index.ts", import.meta.url).href;
  const script = `const{getConfig}=await import(${JSON.stringify(source)});console.log(getConfig().basePath);`;
  for (const path of ["", "/app", "/different/mount"]) {
    const output = execFileSync(
      process.execPath,
      ["--experimental-strip-types", "--input-type=module", "-e", script],
      {
        env: {
          NODE_ENV: "production",
          CHAPCHOO_PUBLIC_URL: `https://example.invalid${path}`,
        },
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    assert.equal(output.trim(), path);
  }
});

void test("development serves the configured mount while distributed builds stay neutral", () => {
  const source = new URL("../../next.config.mjs", import.meta.url).href;
  const script = `const{default:config}=await import(${JSON.stringify(source)});const dev=await config("phase-development-server");const build=await config("phase-production-build");console.log(JSON.stringify({dev:dev.basePath,build:build.basePath??""}));`;
  const output = execFileSync(
    process.execPath,
    ["--input-type=module", "-e", script],
    {
      env: {
        ...process.env,
        NODE_ENV: "development",
        CHAPCHOO_PUBLIC_URL: "https://example.invalid/admin",
      },
      cwd: new URL("../../", import.meta.url),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  assert.deepEqual(JSON.parse(output.trim()), { dev: "/admin", build: "" });
});

void test("login redirect prefix has exactly one owner in each server mode", () => {
  const source = new URL("./index.ts", import.meta.url).href;
  const script = `const{getLoginRedirect}=await import(${JSON.stringify(source)});console.log(getLoginRedirect());`;
  const modes: ("development" | "production")[] = ["development", "production"];
  for (const mode of modes) {
    const output = execFileSync(
      process.execPath,
      ["--input-type=module", "-e", script],
      {
        env: {
          NODE_ENV: mode,
          CHAPCHOO_PUBLIC_URL: "https://example.invalid/admin",
        },
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    assert.equal(
      output.trim(),
      mode === "development" ? "/login" : "/admin/login",
    );
  }
});
