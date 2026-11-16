import assert from "node:assert/strict";
import test from "node:test";
import { chatThinking } from "./chatStatus.ts";
import type { ConversationRecord } from "@chapchoo/web-core/conversation-types";

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
  records: [],
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

void test("a live unanswered request withdraws thinking even while the engine reports running", () => {
  const request: ConversationRecord = {
    key: "request:approval",
    nativeId: "approval",
    kind: "request",
    anchor: { ordinal: 1n, index: 0n },
    updatedOrdinal: 1n,
    content: [],
    locations: [],
    responder: { token: "fixture-only" },
  };
  assert.equal(chatThinking({ ...current, records: [request] }), false);
  assert.equal(
    chatThinking({
      ...current,
      records: [{ ...request, responder: void 0 }],
    }),
    true,
    "Historical requests cannot suppress current work",
  );
  assert.equal(
    chatThinking({
      ...current,
      records: [
        {
          ...request,
          answer: {
            value: { type: "permission", optionId: "allow" },
            delivery: "delivered",
          },
        },
      ],
    }),
    true,
    "An answered request no longer asks the owner for input",
  );
});
