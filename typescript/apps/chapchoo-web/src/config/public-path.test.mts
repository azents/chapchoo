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
