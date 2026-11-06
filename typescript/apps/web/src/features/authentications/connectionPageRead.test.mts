import assert from "node:assert/strict";
import test from "node:test";
import { RpcFailure, SafeRenderFailure } from "@chapchoo/web-core/errors";
import { readConnectionPage } from "./connectionPageRead.ts";

void test("unusable connections URL reads select not found without retry", async () => {
  for (const reason of [
    "DOMAIN_REASON_INVALID_INPUT",
    "DOMAIN_REASON_NOT_FOUND",
    "DOMAIN_REASON_PERMISSION_DENIED",
  ] as const) {
    let calls = 0;
    const result = await readConnectionPage(() => {
      calls++;
      return Promise.reject(new SafeRenderFailure(new RpcFailure(3, reason)));
    });
    assert.equal(result, null);
    assert.equal(calls, 1);
  }
});

void test("connections transport and session failures remain route errors", async () => {
  for (const failure of [
    new RpcFailure(14 /* UNAVAILABLE */, null),
    new RpcFailure(16 /* UNAUTHENTICATED */, "DOMAIN_REASON_INVALID_SESSION"),
    new RpcFailure(13 /* INTERNAL */, "DOMAIN_REASON_INTERNAL"),
    new Error("Unexpected read failure"),
  ]) {
    let calls = 0;
    await assert.rejects(
      readConnectionPage(() => {
        calls++;
        return Promise.reject(failure);
      }),
      (actual) => actual === failure,
    );
    assert.equal(calls, 1);
  }
});

void test("connections read preserves the complete successful SSR input", async () => {
  const input = { snapshot: { page: { items: [] } }, managed: null };
  assert.equal(await readConnectionPage(() => Promise.resolve(input)), input);
});
