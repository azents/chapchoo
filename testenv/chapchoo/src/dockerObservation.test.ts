import assert from "node:assert/strict";
import test from "node:test";
import { runningSuccessor } from "./dockerObservation.ts";
import type { NativeComputer, NativeObservation } from "./dockerObservation.ts";

const previous: NativeComputer = {
  container: "a".repeat(64),
  volume: "saved-original",
  daemon: "daemon:original",
  user: "10001:10001",
  noCopy: true,
};
const next: NativeComputer = { ...previous, container: "b".repeat(64) };
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
