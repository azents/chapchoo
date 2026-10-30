import assert from "node:assert/strict";
import test from "node:test";
import { diagnosticTail, safeMessage, sanitize } from "./safety.ts";

void test("all complete 43-character credentials and padded keys are masked", () => {
  const token = "aB_9-".repeat(8) + "aB_";
  assert.equal(token.length, 43);
  for (const input of [
    token,
    `${token}=`,
    `credential=${token}`,
    `\"${token}\"`,
  ]) {
    const result = sanitize(input);
    assert.equal(result.includes(token), false);
    assert.equal(result.includes("[secret]"), true);
  }
});

void test("fixture passwords, proof URL parameters and bearer headers are masked", () => {
  const token = "K".repeat(43);
  const password = `Aa1!${token}`;
  const result = sanitize(
    `Password ${password}; Bearer ${token}; https://example.invalid/app/signup?token=${token}&x=1; proof?proof=short-proof`,
  );
  for (const secret of [token, password, "short-proof"]) {
    assert.equal(result.includes(secret), false);
  }
  assert.equal(result.includes("[password]"), true);
  assert.equal(result.includes("Bearer [credential]"), true);
});

void test("diagnostic tails omit commands, environment, raw stacks and URLs", () => {
  const token = "L".repeat(43);
  const result = diagnosticTail(
    `$ run --secret argument\nCHAPCHOO_SECRET_KEY=${token}=\nEnvironment: private values\nArguments: private arguments\n    at private.stack (source:1)\n 17: private.rust.stack\nError: cannot connect postgres://fixture:private@localhost/db\nCode: fixture Aa1!${token} Bearer ${token}\n`,
  );
  for (const forbidden of [
    token,
    "run --secret",
    "private values",
    "private arguments",
    "private.stack",
    "private.rust.stack",
    "postgres://",
    "localhost",
    "Aa1!",
  ]) {
    assert.equal(result.includes(forbidden), false);
  }
  assert.equal(result.includes("Error: cannot connect [url]"), true);
  assert.equal(result.includes("[password]"), true);
});

void test("partial boundary lines are never released and diagnostics remain bounded", () => {
  const partial = "Z".repeat(42);
  const result = diagnosticTail(
    `${partial}\nComplete diagnostic\n${partial}`,
    true,
  );
  assert.equal(result, "Complete diagnostic");
  const bounded = diagnosticTail(
    Array.from({ length: 100 }, () => "x".repeat(400)).join("\n") + "\n",
  );
  assert.ok(bounded.length <= 6 * 200 + 5 * 3);
});

void test("reporter messages expose only a sanitized bounded first line", () => {
  const token = "M".repeat(43);
  const result = safeMessage(
    `Failure ${token}\n    at raw stack\nprivate detail`,
  );
  assert.equal(result.includes(token), false);
  assert.equal(result.includes("raw stack"), false);
  assert.equal(result.includes("private detail"), false);
  assert.ok(result.length <= 500);
});
