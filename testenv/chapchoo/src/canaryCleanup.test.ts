import assert from "node:assert/strict";
import test from "node:test";
import { cleanupCanary } from "./canaryCleanup.ts";
import type { CanaryState } from "./canaryCleanup.ts";

const labels = { "chapchoo.test/run": "owned" };
function control(initial: CanaryState | null): {
  inspect: () => Promise<CanaryState | null>;
  stop: () => Promise<void>;
  remove: () => Promise<void>;
  calls: string[];
} {
  let state = initial;
  const calls: string[] = [];
  return {
    calls,
    inspect: () => {
      calls.push("inspect");
      return Promise.resolve(state);
    },
    stop: () => {
      calls.push("stop");
      return Promise.resolve();
    },
    remove: () => {
      calls.push("remove");
      state = null;
      return Promise.resolve();
    },
  };
}
void test("a known canary is reconciled from observed state after Start response loss", async () => {
  for (const running of [true, false]) {
    const native = control({ id: "original", running, labels });
    await cleanupCanary(native, "original", labels);
    assert.deepEqual(
      native.calls,
      running
        ? ["inspect", "stop", "remove", "inspect"]
        : ["inspect", "remove", "inspect"],
    );
  }
  const absent = control(null);
  await cleanupCanary(absent, "original", labels);
  assert.deepEqual(absent.calls, ["inspect"]);
});
void test("canary cleanup refuses changed identity or ownership before mutation", async () => {
  for (const state of [
    { id: "foreign", running: true, labels },
    {
      id: "original",
      running: true,
      labels: { "chapchoo.test/run": "foreign" },
    },
  ]) {
    const native = control(state);
    await assert.rejects(
      cleanupCanary(native, "original", labels),
      /identity changed/,
    );
    assert.deepEqual(native.calls, ["inspect"]);
  }
});
void test("canary cleanup does not remove after unconfirmed Stop", async () => {
  const native = control({ id: "original", running: true, labels });
  native.stop = () => Promise.reject(new Error("Stop response lost"));
  await assert.rejects(
    cleanupCanary(native, "original", labels),
    /Stop response lost/,
  );
  assert.deepEqual(native.calls, ["inspect"]);
});
