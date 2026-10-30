import assert from "node:assert/strict";
import { statSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";
import { preparePrivateOutput, privateOutputRoot } from "./index.ts";

void test("Playwright output uses an owned private parent and exact run child", () => {
  const output = preparePrivateOutput();
  assert.equal(output, join(privateOutputRoot, "run"));
  const stat = statSync(privateOutputRoot);
  assert.equal(stat.isDirectory(), true);
  assert.equal(stat.uid, process.getuid?.());
  assert.equal(stat.mode & 0o777, 0o700);
});
