#!/usr/bin/env node
// Deterministic native-protocol fixture. This executable never contacts a model.
// Select it explicitly as an existing CLI in an isolated test Computer only.
const { createInterface } = require("node:readline");
if (process.argv.includes("--version")) {
  process.stdout.write("codex-cli 0.165.0\n");
  return;
}
if (!process.argv.includes("app-server")) {
  process.stderr.write("Fixture requires the app-server command.\n");
  process.exitCode = 2;
  return;
}
const threadId = "fixture-thread";
const turnId = "fixture-turn";
const model = "fixture-model";
let authenticated = false;
let active = false;
let primaryUsed = false;
let primaryStarted = false;
const helperThreads = new Set();
let cwd = "/workspace";
const turn = (status) => ({ id: turnId, items: [], status, error: null });
const thread = () => ({
  id: threadId,
  preview: "Deterministic acceptance",
  modelProvider: "openai",
  createdAt: 1794240000,
  updatedAt: 1794240000,
  source: "appServer",
  status: {
    type: active ? "active" : "idle",
    ...(active ? { activeFlags: [] } : {}),
  },
  path: null,
  cwd,
  cliVersion: "0.165.0",
  turns: [],
  name: null,
  agentNickname: null,
  agentRole: null,
  gitInfo: null,
});
const write = (message) => process.stdout.write(JSON.stringify(message) + "\n");
const reply = (id, result) => write({ id, result });
const notify = (method, params) => write({ method, params });
const item = (method, value) =>
  notify(method, { threadId, turnId, item: value });
function finish(accepted) {
  if (!active) return;
  const text = accepted
    ? "First assistant output — updated after the typed answer."
    : "Fixture work was cancelled.";
  notify("item/agentMessage/delta", {
    threadId,
    turnId,
    itemId: "assistant-one",
    delta: accepted ? " — updated after the typed answer." : " — cancelled.",
  });
  item("item/completed", {
    type: "agentMessage",
    id: "assistant-one",
    text,
    phase: "commentary",
  });
  if (accepted) {
    item("item/started", {
      type: "agentMessage",
      id: "assistant-two",
      text: "",
      phase: "final_answer",
    });
    notify("item/agentMessage/delta", {
      threadId,
      turnId,
      itemId: "assistant-two",
      delta:
        "Second assistant utterance: the native fixture received the exact typed answers.",
    });
    item("item/completed", {
      type: "agentMessage",
      id: "assistant-two",
      text: "Second assistant utterance: the native fixture received the exact typed answers.",
      phase: "final_answer",
    });
  }
  active = false;
  notify("turn/completed", {
    threadId,
    turn: turn(accepted ? "completed" : "interrupted"),
  });
  notify("thread/status/changed", { threadId, status: { type: "idle" } });
}
function request(id, method, params) {
  switch (method) {
    case "initialize":
      reply(id, {
        userAgent: "chapchoo-deterministic-fixture",
        codexHome: process.env.CODEX_HOME ?? null,
        platformFamily: "unix",
        platformOs: "linux",
      });
      break;
    case "account/login/start":
      if (
        params?.type !== "apiKey" ||
        typeof params.apiKey !== "string" ||
        !params.apiKey
      )
        throw new Error("Fixture requires explicit synthetic key login");
      authenticated = true;
      reply(id, { type: "apiKey" });
      notify("account/login/completed", {
        loginId: null,
        success: true,
        error: null,
      });
      break;
    case "account/read":
      reply(id, {
        account: authenticated ? { type: "apiKey" } : null,
        requiresOpenaiAuth: true,
      });
      break;
    case "account/rateLimits/read":
      reply(id, {
        rateLimits: {
          primary: null,
          secondary: null,
          credits: null,
          planType: null,
        },
        rateLimitsByLimitId: null,
      });
      break;
    case "config/read":
      reply(id, {
        config: {
          model,
          model_provider: "openai",
          approval_policy: "on-request",
          sandbox_mode: "workspace-write",
          features: {},
        },
        origins: {},
        layers: [],
      });
      break;
    case "permissionProfile/list":
      reply(id, { data: [], nextCursor: null });
      break;
    case "model/list":
      reply(id, {
        data: [
          {
            id: model,
            model,
            displayName: "Deterministic fixture",
            description: "No model request is made",
            isDefault: true,
            supportedReasoningEfforts: [
              { reasoningEffort: "medium", description: "Fixture default" },
            ],
            defaultReasoningEffort: "medium",
            inputModalities: ["text"],
            supportsPersonality: false,
            supportsParallelToolCalls: false,
          },
        ],
        nextCursor: null,
      });
      break;
    case "skills/list":
      reply(id, { data: [] });
      break;
    case "skills/extraRoots/set":
    case "config/batchWrite":
    case "thread/name/set":
    case "thread/unsubscribe":
      reply(id, {});
      break;
    case "thread/start":
      if (!authenticated || typeof params?.cwd !== "string")
        throw new Error("Fixture thread was not admitted");
      if (params.ephemeral !== true) {
        if (primaryStarted)
          throw new Error("Fixture primary thread was started twice");
        primaryStarted = true;
      }
      cwd = params.cwd;
      reply(id, {
        thread:
          params.ephemeral === true
            ? { ...thread(), id: `fixture-helper-${helperThreads.size + 1}` }
            : thread(),
        model,
        modelProvider: "openai",
        cwd,
        approvalPolicy: "on-request",
        approvalsReviewer: "user",
        sandbox: {
          type: "workspaceWrite",
          writableRoots: [cwd],
          networkAccess: false,
          excludeTmpdirEnvVar: true,
          excludeSlashTmp: true,
        },
        reasoningEffort: "medium",
        serviceTier: null,
        activePermissionProfile: null,
      });
      if (params.ephemeral === true)
        helperThreads.add(`fixture-helper-${helperThreads.size + 1}`);
      break;
    case "thread/read":
      reply(id, { thread: thread() });
      break;
    case "thread/loaded/list":
      reply(id, { data: [threadId], nextCursor: null });
      break;
    case "thread/list":
      reply(id, { data: [thread()], nextCursor: null });
      break;
    case "thread/turns/list":
    case "thread/items/list":
    case "mcpServerStatus/list":
      reply(id, { data: [], nextCursor: null });
      break;
    case "turn/start":
      // The official adapter creates isolated helper threads for title generation.
      // Their deterministic completion must not reuse the conversation identity.
      if (helperThreads.has(params?.threadId)) {
        const helperTurn = {
          id: `${params.threadId}-turn`,
          items: [
            {
              type: "agentMessage",
              id: "helper-title",
              text: '{"title":"Deterministic conversation"}',
              phase: "final_answer",
            },
          ],
          status: "completed",
          error: null,
        };
        reply(id, { turn: { ...helperTurn, items: [], status: "inProgress" } });
        notify("turn/started", {
          threadId: params.threadId,
          turn: { ...helperTurn, items: [], status: "inProgress" },
        });
        notify("turn/completed", {
          threadId: params.threadId,
          turn: helperTurn,
        });
        break;
      }
      if (
        !authenticated ||
        active ||
        primaryUsed ||
        params?.threadId !== threadId ||
        !Array.isArray(params.input) ||
        params.input.length !== 1 ||
        params.input[0]?.type !== "text" ||
        params.input[0]?.text !==
          "Exercise the deterministic native conversation"
      )
        throw new Error("Fixture turn was not admitted");
      primaryUsed = true;
      active = true;
      reply(id, { turn: turn("inProgress") });
      notify("turn/started", { threadId, turn: turn("inProgress") });
      notify("thread/status/changed", {
        threadId,
        status: { type: "active", activeFlags: [] },
      });
      item("item/started", {
        type: "userMessage",
        id: "native-user",
        clientId: params.clientUserMessageId ?? null,
        content: params.input,
      });
      item("item/completed", {
        type: "userMessage",
        id: "native-user",
        clientId: params.clientUserMessageId ?? null,
        content: params.input,
      });
      item("item/started", {
        type: "agentMessage",
        id: "assistant-one",
        text: "",
        phase: "commentary",
      });
      notify("item/agentMessage/delta", {
        threadId,
        turnId,
        itemId: "assistant-one",
        delta: "First assistant output",
      });
      write({
        id: "fixture-form",
        method: "mcpServer/elicitation/request",
        params: {
          threadId,
          turnId,
          serverName: "deterministic-fixture",
          mode: "form",
          message: "Review the native fixture choices",
          requestedSchema: {
            type: "object",
            properties: {
              channel: {
                type: "string",
                title: "Release channel",
                oneOf: [
                  { const: "preview", title: "Preview" },
                  { const: "stable", title: "Stable" },
                ],
              },
              note: {
                type: "string",
                title: "Confirmation note",
                minLength: 1,
              },
            },
            required: ["channel", "note"],
          },
        },
      });
      break;
    case "turn/interrupt":
      reply(id, {});
      finish(false);
      break;
    default:
      write({
        id,
        error: {
          code: -32601,
          message: "Method is not part of the deterministic fixture",
        },
      });
  }
}
const lines = createInterface({ input: process.stdin, crlfDelay: Infinity });
lines.on("line", (line) => {
  try {
    const value = JSON.parse(line);
    if (!value || typeof value !== "object" || Array.isArray(value))
      throw new Error("Invalid fixture message");
    if (value.id === "fixture-form" && typeof value.method !== "string") {
      const answer = value.result;
      if (
        answer?.action !== "accept" ||
        answer.content?.channel !== "stable" ||
        answer.content?.note !== "typed native answer"
      )
        throw new Error(
          "Native fixture did not receive the exact typed answer",
        );
      finish(true);
    } else if (
      typeof value.method === "string" &&
      (typeof value.id === "string" || typeof value.id === "number")
    ) {
      request(value.id, value.method, value.params);
    } else if (typeof value.method !== "string")
      throw new Error("Unexpected fixture response");
  } catch {
    // Never echo protocol arguments, credentials or received content.
    process.stderr.write(
      "Deterministic native fixture rejected its protocol input.\n",
    );
    process.exitCode = 3;
    lines.close();
    process.stdin.destroy();
  }
});
