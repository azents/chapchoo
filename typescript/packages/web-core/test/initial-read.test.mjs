import assert from "node:assert/strict";
import test from "node:test";
import { status } from "@grpc/grpc-js";
import { loadInitialRead } from "../dist/initial-read.js";
import { RpcFailure } from "../dist/errors.js";

test("initial reads resolve before rendering and execute exactly once", async () => {
  let calls = 0;
  const data = { password: true, emailCode: false, signupEmail: false };
  const result = await loadInitialRead(async () => {
    calls += 1;
    return data;
  });
  assert.equal(calls, 1);
  assert.deepEqual(result, { type: "LOADED", data });
});

test("initial errors keep their typed reason without retry or unsafe source", async () => {
  let calls = 0;
  const failure = new RpcFailure(
    status.PERMISSION_DENIED,
    "DOMAIN_REASON_PERMISSION_DENIED",
    new Error("private native detail"),
  );
  const result = await loadInitialRead(async () => {
    calls += 1;
    throw failure;
  });
  assert.equal(calls, 1);
  assert.deepEqual(result, {
    type: "ERROR",
    reason: "DOMAIN_REASON_PERMISSION_DENIED",
  });
  assert.ok(!JSON.stringify(result).includes("private"));
});

test("infrastructure failures are safe error results, never signed-out success", async () => {
  const diagnostics = [];
  const previous = console.error;
  console.error = (line) => diagnostics.push(line);
  try {
    const result = await loadInitialRead(async () => {
      throw new RpcFailure(
        status.UNAVAILABLE,
        null,
        new Error("private native detail"),
      );
    });
    assert.deepEqual(result, { type: "ERROR", reason: null });
    assert.equal(diagnostics.length, 1);
    assert.ok(!diagnostics[0].includes("private native detail"));
  } finally {
    console.error = previous;
  }
});
