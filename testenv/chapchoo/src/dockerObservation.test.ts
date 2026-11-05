import assert from "node:assert/strict";
import test from "node:test";
import {
  nativeReadinessStatus,
  runningSuccessor,
} from "./dockerObservation.ts";
import type { NativeComputer, NativeObservation } from "./dockerObservation.ts";

const previous: NativeComputer = {
  container: "a".repeat(64),
  volume: "saved-original",
  daemon: "daemon:original",
  user: "10001:10001",
  noCopy: true,
};
const next: NativeComputer = { ...previous, container: "b".repeat(64) };
const removing = {
  Status: "removing",
  Running: false,
  Dead: true,
  Paused: false,
  Restarting: false,
  Error: "",
};
void test("fixture admits a dead old execution only while Docker removes it, without declaring a successor ready", () => {
  const status = nativeReadinessStatus(removing);
  assert.equal(status, "removing");
  assert.equal(
    runningSuccessor(previous, { type: "present", native: previous, status }),
    null,
  );
  assert.throws(() =>
    runningSuccessor(previous, { type: "present", native: next, status }),
  );
});
void test("fixture rejects non-removal death, inconsistent power and native failure without disclosing error text", () => {
  for (const state of [
    { ...removing, Status: "dead" },
    { ...removing, Status: "exited" },
    { ...removing, Status: "running", Running: true },
    { ...removing, Running: true },
    { ...removing, Paused: true },
    { ...removing, Restarting: true },
    { ...removing, Error: "private-native-response" },
  ]) {
    assert.throws(
      () => nativeReadinessStatus(state),
      (error: unknown) =>
        error instanceof Error &&
        error.message.includes("readiness state:") &&
        !error.message.includes("private-native-response"),
    );
  }
});
void test("replacement readiness waits through old-running, absence and created before a distinct verified running successor", () => {
  const sequence: NativeObservation[] = [
    { type: "present", native: previous, status: "running" },
    { type: "present", native: previous, status: "exited" },
    { type: "present", native: previous, status: "removing" },
    { type: "absent", daemon: previous.daemon, volume: previous.volume },
    { type: "present", native: next, status: "created" },
    { type: "present", native: next, status: "running" },
  ];
  assert.deepEqual(
    sequence.map((observation) => runningSuccessor(previous, observation)),
    [null, null, null, null, null, next],
  );
});
void test("replacement readiness never hides changed daemon, original storage or a failed successor", () => {
  for (const observation of [
    { type: "absent", daemon: "daemon:other", volume: previous.volume },
    {
      type: "absent",
      daemon: previous.daemon,
      volume: "replacement-empty-volume",
    },
    {
      type: "present",
      native: { ...next, volume: "replacement-empty-volume" },
      status: "running",
    },
    { type: "present", native: next, status: "exited" },
    { type: "present", native: next, status: "removing" },
  ] satisfies NativeObservation[]) {
    assert.throws(() => runningSuccessor(previous, observation));
  }
});
