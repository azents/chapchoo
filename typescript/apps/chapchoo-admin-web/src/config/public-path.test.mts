import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import test from "node:test";

void test("public build prefix matches runtime or fails visibly", () => {
  const source = new URL("./index.ts", import.meta.url).href;
  const buildScript = `const{getBuildConfig}=await import(${JSON.stringify(source)});console.log(getBuildConfig().basePath);`;
  const runtimeScript = `const{getConfig}=await import(${JSON.stringify(source)});console.log(getConfig().basePath);`;
  const publicUrl = "https://example.invalid/chapchoo";
  const build = execFileSync(
    process.execPath,
    ["--experimental-strip-types", "--input-type=module", "-e", buildScript],
    {
      env: { NODE_ENV: "production", CHAPCHOO_PUBLIC_URL: publicUrl },
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  assert.equal(build.trim(), "/chapchoo");
  const runtime = execFileSync(
    process.execPath,
    ["--experimental-strip-types", "--input-type=module", "-e", runtimeScript],
    {
      env: {
        NODE_ENV: "production",
        CHAPCHOO_PUBLIC_URL: publicUrl,
        CHAPCHOO_COMPILED_BASE_PATH: "/chapchoo",
      },
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  assert.equal(runtime.trim(), "/chapchoo");
  for (const env of [
    { NODE_ENV: "production", CHAPCHOO_PUBLIC_URL: publicUrl },
    {
      NODE_ENV: "production",
      CHAPCHOO_PUBLIC_URL: publicUrl,
      CHAPCHOO_COMPILED_BASE_PATH: "/other",
    },
  ] satisfies NodeJS.ProcessEnv[]) {
    assert.throws(() =>
      execFileSync(
        process.execPath,
        [
          "--experimental-strip-types",
          "--input-type=module",
          "-e",
          runtimeScript,
        ],
        { env, stdio: ["ignore", "pipe", "pipe"] },
      ),
    );
  }
});
