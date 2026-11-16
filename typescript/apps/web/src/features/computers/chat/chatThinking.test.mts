import assert from "node:assert/strict";
import test from "node:test";
import { chatThinking } from "./chatStatus.ts";

const current: Parameters<typeof chatThinking>[0] = {
  computer: {
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
  },
  header: {
    agentId: "agent",
    conversationId: "conversation",
    state: "ready",
    foreground: "working",
  },
  read: "ready",
  stale: false,
  authorized: true,
  stop: "idle",
};

void test("thinking requires positively observed current native work", () => {
  assert.equal(chatThinking(current), true);
  for (const foreground of ["idle", "requiresAction", "unavailable"] as const) {
    assert.equal(
      chatThinking({
        ...current,
        header: { ...current.header, foreground },
      }),
      false,
    );
  }
  for (const state of ["notStarted", "loading"] as const) {
    assert.equal(
      chatThinking({ ...current, header: { ...current.header, state } }),
      false,
    );
  }
});

void test("observation gaps and host lifecycle changes withdraw thinking without changing retained work", () => {
  for (const patch of [
    { authorized: false },
    { stale: true },
    { read: "loading" },
    { read: "error" },
    { computer: { ...current.computer, power: "unknown" } },
    { computer: { ...current.computer, power: "stopped" } },
    { computer: { ...current.computer, connector: "disconnected" } },
    { computer: { ...current.computer, intent: "stopped" } },
    { computer: { ...current.computer, phase: "accepted" } },
    { computer: { ...current.computer, phase: "retiring" } },
    { computer: { ...current.computer, phase: "blocked" } },
  ] satisfies Partial<Parameters<typeof chatThinking>[0]>[]) {
    assert.equal(chatThinking({ ...current, ...patch }), false);
  }
  assert.equal(current.header.foreground, "working");
  assert.equal(chatThinking(current), true);
  assert.equal(
    chatThinking({
      ...current,
      computer: {
        ...current.computer,
        management: "disconnected",
      },
    }),
    true,
    "Independent host control does not erase connected native work",
  );
});

void test("stop submission or uncertainty withdraws thinking; rejected stop retains observed work", () => {
  for (const stop of ["pending", "published", "uncertain"] as const) {
    assert.equal(chatThinking({ ...current, stop }), false);
  }
  assert.equal(chatThinking({ ...current, stop: "rejected" }), true);
});
