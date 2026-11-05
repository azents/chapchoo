import assert from "node:assert/strict";
import test from "node:test";
import {
  readSelection,
  serverSearchValues,
} from "@chapchoo/web-core/page-state";
import {
  agentCursor,
  agentCursorKey,
  agentNavigationHref,
  agentNavigationItems,
  routeAgentHandle,
} from "./agentSelection.ts";
import type { AgentView } from "@chapchoo/web-core/computer";

const current: AgentView = {
  id: "current",
  teamId: "team-a",
  name: "Current",
  handle: "current",
  paletteKey: "sage",
  profileImage: { id: "image", assetKey: "03-corgi" },
  computer: {
    operation: "start",
    requestId: "request",
    power: "running",
    lastKnownPower: "running",
    phase: "completed",
    intent: "running",
    management: "connected",
    connector: "connected",
    saved: "verified",
    observedAt: null,
    actions: [],
    forceRemoveAvailable: false,
    startDelayed: false,
    preparationRetryAvailable: false,
  },
};

void test("route Agent remains selected outside the bounded page without duplicate or foreign-Team rows", () => {
  const other = { ...current, id: "other", handle: "other" };
  assert.deepEqual(
    agentNavigationItems([other], current, "team-a", "current"),
    [current, other],
  );
  assert.deepEqual(
    agentNavigationItems([current], current, "team-a", "current"),
    [current],
  );
  assert.deepEqual(
    agentNavigationItems([other], current, "team-b", "current"),
    [],
  );
  assert.deepEqual(agentNavigationItems([other], current, "team-a", "other"), [
    other,
  ]);
  assert.deepEqual(agentNavigationItems([other], null, "team-a", "current"), [
    other,
  ]);
});

void test("route selection decodes handles like server params and rejects malformed routes", () => {
  assert.equal(routeAgentHandle("/teams/my/agents/%63urrent/chat"), "current");
  assert.equal(routeAgentHandle("/teams/my/agents/current"), "current");
  for (const route of [
    "/teams/my/agents",
    "/teams/my/new-agent",
    "/teams/my/agents/%FF",
    "/teams/my/agents/invalid%2Fhandle",
  ]) {
    assert.equal(routeAgentHandle(route), null);
  }
});

void test("sidebar URL cursor is bounded, separate from primary after and agrees on initial server/client selection", () => {
  for (const value of [null, "", "x".repeat(129)]) {
    assert.equal(agentCursor.parse(value), null);
  }
  const server = serverSearchValues({
    agentAfter: ["cursor", "ignored"],
    after: "main",
  });
  const browser = new URLSearchParams(
    "agentAfter=cursor&agentAfter=ignored&after=main",
  );
  assert.equal(readSelection(server, agentCursorKey, agentCursor), "cursor");
  assert.equal(readSelection(browser, agentCursorKey, agentCursor), "cursor");
  assert.equal(
    agentNavigationHref("/mount/teams/my/agents?after=main#list", "cursor"),
    "/mount/teams/my/agents?after=main&agentAfter=cursor#list",
  );
  assert.equal(
    agentNavigationHref(
      "/mount/teams/my/agents?after=main&agentAfter=cursor",
      null,
    ),
    "/mount/teams/my/agents?after=main",
  );
  for (const href of [
    "/mount/teams/my",
    "/mount/teams/my/agents",
    "/mount/teams/my/new-agent",
    "/mount/teams/my/agents/current/chat",
  ]) {
    assert.equal(
      new URL(
        agentNavigationHref(href, "cursor"),
        "https://example.test",
      ).searchParams.get(agentCursorKey),
      "cursor",
    );
  }
});
