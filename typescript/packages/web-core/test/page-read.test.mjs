import assert from "node:assert/strict";
import test from "node:test";
import { PageReadScope, requiredPageRead } from "../dist/page-read.js";
import {
  RpcFailure,
  SafeRenderFailure,
  invalidSession,
} from "../dist/errors.js";
import { status } from "@grpc/grpc-js";
const user = {
  id: "user",
  email: "user@example.test",
  isAdmin: false,
  createdAt: { seconds: 1n, nanos: 0 },
};

test("required snapshot reads resolve once and preserve typed failure without replay", async () => {
  let calls = 0;
  const data = { available: true };
  assert.equal(
    await requiredPageRead(() => {
      calls++;
      return Promise.resolve(data);
    }),
    data,
  );
  assert.equal(calls, 1);
  await assert.rejects(
    requiredPageRead(() => {
      calls++;
      return Promise.reject(
        new RpcFailure(
          status.PERMISSION_DENIED,
          "DOMAIN_REASON_PERMISSION_DENIED",
        ),
      );
    }),
    (error) =>
      error instanceof SafeRenderFailure &&
      error.reason === "DOMAIN_REASON_PERMISSION_DENIED",
  );
  assert.equal(calls, 2);
});

test("one request scope validates identity once and lends only its own token to reads", async () => {
  let calls = 0;
  const scope = new PageReadScope(
    "x".repeat(43),
    () => {
      calls++;
      return Promise.resolve(user);
    },
    false,
  );
  const [a, b] = await Promise.all([scope.identity(), scope.identity()]);
  assert.equal(a, b);
  assert.equal(calls, 1);
  assert.equal(await scope.read((token) => Promise.resolve(token.length)), 43);
  assert.equal(calls, 1);
  assert.equal(JSON.stringify(scope).includes("x".repeat(43)), false);
});
test("missing or invalid sessions cannot dispatch primary business reads", async () => {
  let dispatched = false;
  for (const scope of [
    new PageReadScope(null, () => Promise.resolve(user), false),
    new PageReadScope(
      "x".repeat(43),
      () => Promise.reject(invalidSession()),
      false,
    ),
  ]) {
    assert.equal(await scope.identity(), null);
    await assert.rejects(
      scope.read(() => {
        dispatched = true;
        return Promise.resolve();
      }),
      (error) =>
        error instanceof RpcFailure &&
        error.reason === "DOMAIN_REASON_INVALID_SESSION",
    );
  }
  assert.equal(dispatched, false);
});
test("required page failures retain a safe render wrapper without exposing the native cause", async () => {
  const previous = console.error;
  const logs = [];
  console.error = (line) => logs.push(line);
  try {
    await assert.rejects(
      requiredPageRead(() =>
        Promise.reject(
          new RpcFailure(
            status.UNAVAILABLE,
            null,
            new Error("credential-private"),
          ),
        ),
      ),
      (error) => {
        assert.ok(error instanceof SafeRenderFailure);
        assert.equal(error.cause, undefined);
        assert.equal(
          JSON.stringify(error).includes("credential-private"),
          false,
        );
        return true;
      },
    );
    assert.equal(logs.length, 1);
    assert.ok(!logs[0].includes("credential-private"));
  } finally {
    console.error = previous;
  }
});
