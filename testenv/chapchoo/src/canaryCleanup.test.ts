import assert from "node:assert/strict";
import test from "node:test";
import {
  cleanupCanary,
  cleanupCanaryVolume,
  recoverCanaryId,
} from "./canaryCleanup.ts";
import type { CanaryState, CanaryVolumeState } from "./canaryCleanup.ts";

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
void test("lost canary creation is correlated by reserved name, full ID and run labels", () => {
  const state = { id: "a".repeat(64), name: "/owned-container", labels };
  assert.equal(recoverCanaryId(state, "owned-container", labels), state.id);
  for (const foreign of [
    { ...state, id: "short" },
    { ...state, name: "/foreign" },
    { ...state, labels: { "chapchoo.test/run": "foreign" } },
  ]) {
    assert.throws(
      () => recoverCanaryId(foreign, "owned-container", labels),
      /unverified/,
    );
  }
});

void test("a lost CreateVolume response permits exact observed canary cleanup, not recreation", async () => {
  let state: CanaryVolumeState | null = {
    name: "owned-volume",
    driver: "local",
    scope: "local",
    labels,
  };
  const calls: string[] = [];
  await cleanupCanaryVolume(
    {
      inspect: () => {
        calls.push("inspect");
        return Promise.resolve(state);
      },
      remove: () => {
        calls.push("remove");
        state = null;
        return Promise.resolve();
      },
    },
    "owned-volume",
    labels,
  );
  assert.deepEqual(calls, ["inspect", "remove", "inspect"]);
});

void test("canary volume cleanup refuses foreign identity and never repeats an uncertain Remove", async () => {
  const original = {
    name: "owned-volume",
    driver: "local",
    scope: "local",
    labels,
  };
  for (const state of [
    { ...original, name: "foreign" },
    { ...original, driver: "foreign" },
    { ...original, scope: "global" },
    { ...original, labels: { "chapchoo.test/run": "foreign" } },
  ]) {
    let removes = 0;
    await assert.rejects(
      cleanupCanaryVolume(
        {
          inspect: () => Promise.resolve(state),
          remove: () => {
            removes += 1;
            return Promise.resolve();
          },
        },
        "owned-volume",
        labels,
      ),
      /unverified/,
    );
    assert.equal(removes, 0);
  }
  let removes = 0;
  await assert.rejects(
    cleanupCanaryVolume(
      {
        inspect: () => Promise.resolve(original),
        remove: () => {
          removes += 1;
          return Promise.reject(new Error("Remove response lost"));
        },
      },
      "owned-volume",
      labels,
    ),
    /Remove response lost/,
  );
  assert.equal(removes, 1);
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
