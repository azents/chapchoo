import assert from "node:assert/strict";
import test from "node:test";
import {
  agent,
  agents,
  actionNumber,
  creation,
  historicalCreation,
  inventory,
  providers,
  resources,
  profileImages,
} from "../dist/computer.js";
import { computerCopy } from "../dist/computer-messages.js";
import { validAgentPalette } from "../dist/agent-profile.js";

const computer = {
  id: "computer",
  kind: 1,
  desired: 1,
  requestId: "request",
  action: 2,
  phase: 5,
  power: 3,
  lastKnownPower: 1,
  management: 2,
  connector: 2,
  powerObservedAt: { seconds: 1793577600n, nanos: 0 },
  savedEnvironment: 1,
  actions: [
    { action: 2, available: false, blockedReason: 5 },
    { action: 5, available: true },
  ],
  forceRemoveAvailable: false,
  startDelayed: false,
  preparationRetryAvailable: false,
};
const wire = {
  id: "agent",
  teamId: "team",
  name: "Companion",
  computer,
  handle: "companion",
  profileImageId: "image",
  profileImageAssetKey: "03-corgi",
  paletteKey: "sage",
};

test("saved profile decoding preserves opaque references and rejects unsafe presentation", () => {
  const saved = agent(wire);
  assert.equal(saved.handle, "companion");
  assert.deepEqual(saved.profileImage, { id: "image", assetKey: "03-corgi" });
  assert.equal(saved.paletteKey, "sage");
  assert.equal(saved.authentication, null);
  const selected = agent({
    ...wire,
    authentication: {
      authenticationId: "credential",
      name: "Development",
      kind: 2,
      requestId: "selected",
      selectedAt: { seconds: 1n, nanos: 0 },
    },
  });
  assert.deepEqual(selected.authentication, {
    authenticationId: "credential",
    name: "Development",
    kind: "openaiApiKey",
    requestId: "selected",
    selectedAt: 1000,
  });
  for (const bad of [
    { handle: "Invalid" },
    { handle: "a\n" },
    { profileImageId: "" },
    { profileImageAssetKey: "../secret" },
    { profileImageAssetKey: "03-corgi\n" },
    { paletteKey: "unknown" },
  ])
    assert.throws(() => agent({ ...wire, ...bad }));
});
test("read-only historical results distinguish absent observation from accepted and removed", () => {
  assert.deepEqual(historicalCreation({}), { type: "notObserved" });
  assert.deepEqual(
    historicalCreation({
      present: { assignedAgentId: "opaque-agent", teamId: "opaque-team" },
    }),
    { type: "present", agentId: "opaque-agent", teamId: "opaque-team" },
  );
  assert.deepEqual(
    historicalCreation({ removed: { assignedAgentId: "opaque-agent" } }),
    { type: "removed", agentId: "opaque-agent" },
  );
  assert.throws(() =>
    historicalCreation({ present: { assignedAgentId: "", teamId: "team" } }),
  );
  assert.throws(() =>
    historicalCreation({
      present: { assignedAgentId: "agent", teamId: "team" },
      removed: { assignedAgentId: "agent" },
    }),
  );
});
test("profile catalogue projects opaque IDs and safe logical assets in bounded pages", () => {
  assert.deepEqual(
    profileImages({
      items: [
        { id: "opaque-image", internalName: "Corgi", assetKey: "03-corgi" },
      ],
      nextAfter: "opaque-image",
    }),
    {
      items: [
        { id: "opaque-image", internalName: "Corgi", assetKey: "03-corgi" },
      ],
      nextAfter: "opaque-image",
    },
  );
  for (const assetKey of [
    "../secret",
    "https://external.invalid/image",
    "file/name",
    "03-corgi\n",
    "",
  ]) {
    assert.throws(() =>
      profileImages({
        items: [{ id: "image", internalName: "Name", assetKey }],
      }),
    );
  }
  assert.throws(() =>
    profileImages({
      items: Array.from({ length: 21 }, () => ({
        id: "image",
        internalName: "Name",
        assetKey: "03-corgi",
      })),
    }),
  );
  assert.equal(validAgentPalette("sage"), true);
  assert.equal(validAgentPalette("Sage"), false);
  assert.equal(validAgentPalette("unknown"), false);
});

test("Computer DTO preserves unknown power separately from last-known evidence", () => {
  const result = agent(wire);
  assert.equal(result.teamId, "team");
  assert.equal(result.computer.operation, "start");
  assert.equal(result.computer.power, "unknown");
  assert.equal(result.computer.lastKnownPower, "running");
  assert.equal(result.computer.management, "disconnected");
  assert.equal(result.computer.observedAt, 1793577600000);
  assert.deepEqual(result.computer.actions, [
    { action: "start", available: false, reason: "powerUnknown" },
    { action: "delete", available: true, reason: null },
  ]);
  assert.doesNotThrow(() => JSON.stringify(result));
});
test("Browser DTO is a projection and does not copy internal fields", () => {
  const result = agent({
    ...wire,
    ownerUserId: "private-owner",
    computer: {
      ...computer,
      executionId: "private-execution",
      connectionId: "private-route",
      token: "private-secret",
    },
  });
  const serialized = JSON.stringify(result);
  assert.equal(serialized.includes("private-"), false);
  assert.equal("id" in result.computer, false);
});
test("Malformed required Computer projections fail rather than inventing a state", () => {
  for (const bad of [
    { ...wire, computer: void 0 },
    { ...wire, id: "" },
    { ...wire, teamId: "" },
    { ...wire, teamId: void 0 },
    { ...wire, computer: { ...computer, power: 0 } },
    { ...wire, computer: { ...computer, action: 0 } },
    { ...wire, computer: { ...computer, phase: 99 } },
    {
      ...wire,
      computer: { ...computer, powerObservedAt: { seconds: 1n, nanos: -1 } },
    },
  ])
    assert.throws(() => agent(bad));
});
test("Creation replay distinguishes an existing Agent from a removed receipt", () => {
  assert.equal(creation({ agent: wire }).type, "accepted");
  assert.deepEqual(creation({ removed: { assignedAgentId: "previous" } }), {
    type: "removed",
  });
  assert.throws(() =>
    creation({ agent: wire, removed: { assignedAgentId: "previous" } }),
  );
  assert.throws(() => creation({}));
});
test("Inventory remains independent, redacted and retains cleanup visibility", () => {
  const result = inventory({
    items: [
      {
        ...computer,
        residual: true,
        cleanupCompletedAt: { seconds: 1793577600n, nanos: 0 },
        agentName: "private-name",
      },
    ],
    nextAfter: "cursor",
  });
  assert.equal(result.items[0].residual, true);
  assert.equal(result.items[0].cleanupCompletedAt, 1793577600000);
  assert.equal(JSON.stringify(result).includes("private-name"), false);
  assert.equal(result.nextAfter, "cursor");
  assert.equal(
    providers({
      items: [
        {
          id: "provider",
          name: "Community",
          management: 2,
          kind: 1,
          capabilities: {
            create: true,
            start: true,
            stop: true,
            restart: true,
          },
        },
      ],
    }).items[0].management,
    "disconnected",
  );
});
test("Provider capability and safe preparation retry are explicit redacted projections", () => {
  for (const canCreate of [true, false]) {
    const result = providers({
      items: [
        {
          id: "provider",
          name: "Community",
          management: 1,
          kind: 2,
          capabilities: {
            create: canCreate,
            start: true,
            stop: true,
            restart: true,
          },
          ownerUserId: "private-owner",
        },
      ],
    });
    assert.equal(result.items[0].canCreate, canCreate);
    assert.equal(result.items[0].kind, "docker");
    assert.equal(result.items[0].management, "connected");
    assert.equal(JSON.stringify(result).includes("private-owner"), false);
  }
  assert.throws(() =>
    providers({
      items: [{ id: "provider", name: "Community", management: 1 }],
    }),
  );
  const result = agent({
    ...wire,
    computer: {
      ...computer,
      preparationRetryAvailable: true,
      executionId: "private-execution",
    },
  });
  assert.equal(result.computer.preparationRetryAvailable, true);
  assert.equal(JSON.stringify(result).includes("private-execution"), false);
});
test("List limits and action mapping enforce the browser boundary", () => {
  assert.throws(() =>
    agents({ items: Array.from({ length: 21 }, () => wire) }),
  );
  assert.deepEqual(
    ["start", "stop", "restart", "delete"].map(actionNumber),
    [2, 3, 4, 5],
  );
});
test("Resource inventory retains exact identities without leaking bootstrap data", () => {
  const resource = {
    identity: {
      kind: "PVC",
      namespace: "managed",
      name: "original",
      nativeScopeKind: "",
      nativeScope: "",
    },
    uid: "original-uid",
    executionId: undefined,
    presence: "UNKNOWN",
    observedAt: undefined,
    data: { token: "private-bootstrap" },
  };
  const result = resources({ items: [resource], nextAfter: resource.identity });
  assert.equal(result.items[0].uid, "original-uid");
  assert.equal(result.items[0].presence, "unknown");
  assert.equal(JSON.stringify(result).includes("private-bootstrap"), false);
  assert.throws(() =>
    resources({ items: [{ ...resource, presence: "DELETED" }] }),
  );
});
test("Docker inventory preserves daemon and full IDs while rejecting mixed native metadata", () => {
  const identity = {
    kind: "DOCKER_CONTAINER",
    namespace: "",
    name: "execution",
    nativeScopeKind: "DOCKER_DAEMON",
    nativeScope: "daemon-" + "x".repeat(600),
  };
  const container = {
    identity,
    dockerContainerId: "a".repeat(64),
    presence: "ABSENT",
    endpoint: "private-endpoint",
    labels: { token: "private-bootstrap" },
  };
  const result = resources({ items: [container], nextAfter: identity });
  assert.deepEqual(result.nextAfter, identity);
  assert.equal(result.items[0].dockerContainerId, container.dockerContainerId);
  assert.equal(result.items[0].uid, null);
  assert.equal(JSON.stringify(result).includes("private-"), false);
  const volume = {
    identity: { ...identity, kind: "DOCKER_VOLUME", name: "original" },
    presence: "UNKNOWN",
    dockerVolumeCreatedAt: "2026-11-03T06:45:15Z",
  };
  assert.equal(
    resources({ items: [volume] }).items[0].dockerVolumeCreatedAt,
    volume.dockerVolumeCreatedAt,
  );
  for (const invalid of [
    { ...container, uid: "fake-kubernetes" },
    { ...container, dockerContainerId: "a".repeat(12) },
    { ...container, dockerVolumeCreatedAt: volume.dockerVolumeCreatedAt },
    { ...volume, dockerContainerId: container.dockerContainerId },
    { ...container, identity: { ...identity, namespace: "managed" } },
    { ...container, identity: { ...identity, nativeScopeKind: "" } },
    { ...container, identity: { ...identity, nativeScope: "bad daemon" } },
    { ...container, identity: { ...identity, nativeScope: "é".repeat(513) } },
    { ...container, identity: { ...identity, kind: "UNKNOWN" } },
    {
      ...container,
      identity: { ...identity, kind: "PVC", namespace: "managed" },
    },
  ])
    assert.throws(() => resources({ items: [invalid] }));
});
test("Desired intent remains independent from actual power", () => {
  const result = agent({ ...wire, computer: { ...computer, desired: 3 } });
  assert.equal(result.computer.intent, "deleting");
  assert.equal(result.computer.power, "unknown");
});
test("Computer copy keys stay aligned and retain consequential warnings in every locale", () => {
  const english = computerCopy("en-US");
  for (const locale of ["ko-KR", "ja-JP", "fr-FR"]) {
    const copy = computerCopy(locale);
    assert.deepEqual(Object.keys(copy).sort(), Object.keys(english).sort());
    for (const key of Object.keys(english)) assert.ok(copy[key].length > 0);
    assert.notEqual(copy.forceWarning, english.forceWarning);
    assert.notEqual(copy.stopWarning, english.stopWarning);
  }
});
