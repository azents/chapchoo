import assert from "node:assert/strict";
import test from "node:test";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { chatStatus } from "./chatStatus.ts";
import { activityStatus, chatCopy } from "./copy.ts";
import type { ComputerView } from "@chapchoo/web-core/computer";

const computer: ComputerView = {
  operation: "start",
  intent: "running",
  requestId: "request",
  power: "running",
  lastKnownPower: "running",
  phase: "completed",
  management: "connected",
  connector: "connected",
  saved: "verified",
  observedAt: 1,
  actions: [],
  forceRemoveAvailable: false,
  startDelayed: false,
  preparationRetryAvailable: false,
};
const copy = chatCopy("ko-KR");
const base: Parameters<typeof chatStatus>[0] = {
  computer,
  activity: "working",
  foreground: "working",
  read: "ready",
  stale: false,
  computerCopy: computerCopy("ko-KR"),
  activityLabel: (activity) => activityStatus(copy, activity),
};

void test("retained native state survives observation gaps with only a refreshing flag", () => {
  for (const gap of [
    { computer: { ...computer, power: "unknown" as const } },
    { computer: { ...computer, connector: "disconnected" as const } },
    { read: "error" as const },
    { stale: true, error: "Old read error" },
  ]) {
    assert.deepEqual(chatStatus({ ...base, ...gap }), {
      label: "작업 중",
      refreshing: true,
    });
  }
  assert.deepEqual(chatStatus(base), { label: "작업 중", refreshing: false });
});

void test("known Computer lifecycle and stopped power supersede retained native work", () => {
  for (const [value, label] of [
    [{ ...computer, power: "stopped" as const }, base.computerCopy.stopped],
    [{ ...computer, phase: "accepted" as const }, base.computerCopy.starting],
    [
      {
        ...computer,
        operation: "stop" as const,
        intent: "stopped" as const,
        phase: "accepted" as const,
      },
      base.computerCopy.retiring,
    ],
    [
      {
        ...computer,
        operation: "restart" as const,
        phase: "retiring" as const,
      },
      base.computerCopy.checkingRestart,
    ],
  ] satisfies [ComputerView, string][]) {
    assert.deepEqual(chatStatus({ ...base, computer: value }), {
      label,
      refreshing: false,
    });
  }
});

void test("initial absence invents no previous state and confirmed failures are not loading", () => {
  assert.deepEqual(
    chatStatus({ ...base, foreground: "unavailable", activity: "unavailable" }),
    { label: "", refreshing: true },
  );
  for (const activity of [
    "startFailed",
    "sourceUnavailable",
    "retirementUnknown",
    "deliveryFailed",
    "authenticationRequired",
  ] as const) {
    const value = chatStatus({
      ...base,
      foreground: "unavailable",
      activity,
      read: "error",
    });
    assert.equal(value.label, activityStatus(copy, activity));
    assert.equal(value.refreshing, false);
  }
});
