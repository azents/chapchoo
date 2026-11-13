import type { ChatInput } from "../types";
import type { User } from "@chapchoo/web-core/browser";

export const containerUser: User = {
  id: "fixture-user",
  email: "owner@example.invalid",
  isAdmin: false,
  createdAt: { seconds: 0n, nanos: 0 },
  display: { language: "en", region: "en-US", timeZone: "UTC" },
};
export const containerInput: ChatInput = {
  basePath: "",
  initial: {
    type: "detail",
    renderedAt: 1,
    identity: containerUser.email,
    teamId: "team",
    teamHandle: "my",
    authenticationSnapshots: [],
    agent: {
      id: "agent",
      teamId: "team",
      name: "Corgi",
      handle: "corgi",
      profileImage: { id: "corgi", assetKey: "03-corgi" },
      paletteKey: "teal",
      authentication: null,
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
    },
  },
  initialConversation: {
    header: {
      agentId: "agent",
      conversationId: "conversation",
      state: "ready",
      foreground: "idle",
      source: {
        agentId: "agent",
        conversationId: "conversation",
        snapshotId: "snapshot",
      },
    },
    records: [],
    inputs: [],
    renderedAt: 1,
  },
  initialMessages: {
    agentId: "agent",
    conversationId: "conversation",
    messages: [],
    through: 0n,
    window: { type: "latest" },
    renderedAt: 1,
  },
};
export const containerFailure: ChatInput = {
  ...containerInput,
  initialConversation: {
    ...containerInput.initialConversation,
    inputs: [
      {
        id: "failed-input",
        requestId: "failed-request",
        conversationId: "conversation",
        acceptedAt: 1n,
        text: "Retained failed queue item",
        observation: "failed",
      },
    ],
    latestInput: { requestId: "failed-request", observation: "failed" },
  },
};
