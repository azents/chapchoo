#!/usr/bin/env node
// Deterministic native-protocol fixture. This executable never contacts a model.
// Select it explicitly as an existing CLI in an isolated test Computer only.
const { createInterface } = require("node:readline");
const { existsSync, writeFileSync } = require("node:fs");
const { join } = require("node:path");
if (process.argv.includes("--version")) {
  process.stdout.write("codex-cli 0.165.0\n");
  return;
}
if (!process.argv.includes("app-server")) {
  process.stderr.write("Fixture requires the app-server command.\n");
  process.exitCode = 2;
  return;
}
const { Client } = require("@modelcontextprotocol/sdk/client/index.js");
const {
  StreamableHTTPClientTransport,
} = require("@modelcontextprotocol/sdk/client/streamableHttp.js");

// The SDK's public request method selects this fixture's explicit MCP profile;
// its supported transport owns framing, sessions, parsing and HTTP lifetime.
class FixtureMcpClient extends Client {
  request(value, schema, options) {
    return super.request(
      value.method === "initialize"
        ? {
            ...value,
            params: { ...value.params, protocolVersion: "2025-06-18" },
          }
        : value,
      schema,
      options,
    );
  }
}
let mcp = null;
let mcpTools = [];
if (!process.env.CODEX_HOME)
  throw new Error("Fixture requires its explicit saved native state");
const retainedThread = join(
  process.env.CODEX_HOME,
  "conversation-fixture-thread",
);
async function connectMcp(servers) {
  const definition = servers?.chapchoo;
  if (
    !definition ||
    typeof definition.url !== "string" ||
    typeof definition.http_headers?.Authorization !== "string"
  )
    throw new Error("Fixture requires its explicit server-tools declaration");
  const client = new FixtureMcpClient(
    { name: "chapchoo-native-fixture", version: "1.0.0" },
    { capabilities: {} },
  );
  const transport = new StreamableHTTPClientTransport(new URL(definition.url), {
    requestInit: { headers: definition.http_headers, redirect: "error" },
    reconnectionOptions: {
      initialReconnectionDelay: 1000,
      maxReconnectionDelay: 1000,
      reconnectionDelayGrowFactor: 1,
      maxRetries: 0,
    },
  });
  mcp = client;
  await client.connect(transport);
  const catalog = await client.listTools();
  if (catalog.tools.length !== 1 || catalog.tools[0]?.name !== "send_message")
    throw new Error("Fixture requires its exact server-tools catalog");
  mcpTools = catalog.tools;
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
async function request(id, method, params) {
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
    case "thread/resume":
      if (!authenticated || typeof params?.cwd !== "string")
        throw new Error("Fixture thread was not admitted");
      if (params.ephemeral !== true) {
        if (
          params.config?.code_mode?.direct_only_tool_namespaces?.length !== 1 ||
          params.config.code_mode.direct_only_tool_namespaces[0] !==
            "mcp__chapchoo" ||
          params.config?.features?.code_mode_only_strict_3p_tools !== false
        )
          throw new Error(
            "Fixture requires directly exposed conversation tools",
          );
        if (primaryStarted)
          throw new Error("Fixture primary thread was started twice");
        if (
          method === "thread/resume" &&
          (params.threadId !== threadId || !existsSync(retainedThread))
        )
          throw new Error("Fixture exact retained thread is unavailable");
        primaryStarted = true;
        await connectMcp(params.config?.mcp_servers);
        if (method === "thread/start")
          writeFileSync(retainedThread, "fixture-thread\n", {
            flag: "wx",
            mode: 0o600,
          });
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
      reply(id, { data: [], nextCursor: null });
      break;
    case "mcpServerStatus/list":
      reply(id, {
        data: mcp
          ? [
              {
                name: "chapchoo",
                runtimeStatus: "connected",
                pluginId: null,
                httpOrigin: null,
                serverInfo: { name: "chapchoo", version: "fixture" },
                serverCapabilities: { tools: {} },
                toolListMeta: null,
                tools: Object.fromEntries(
                  mcpTools.map((tool) => [tool.name, tool]),
                ),
                toolsError: null,
                resources: [],
                resourceTemplates: [],
                authStatus: "bearerToken",
              },
            ]
          : [],
        nextCursor: null,
      });
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
      // This explicit SDK call, not an assistant/final event, publishes Chat.
      if (!mcp) throw new Error("Fixture MCP owner is unavailable");
      const publication = {
        type: "mcpToolCall",
        id: "publication-one",
        server: "chapchoo",
        tool: "send_message",
        arguments: { text: "Intentional fixture publication." },
        status: "inProgress",
        result: null,
        error: null,
        durationMs: null,
      };
      item("item/started", publication);
      const publicationStart = performance.now();
      const result = await mcp.callTool({
        name: publication.tool,
        arguments: publication.arguments,
      });
      if (result.isError)
        throw new Error("Fixture publication was not accepted");
      item("item/completed", {
        ...publication,
        status: "completed",
        result,
        durationMs: Math.round(performance.now() - publicationStart),
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
lines.on("line", async (line) => {
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
      await request(value.id, value.method, value.params);
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
lines.once("close", () => {
  void mcp?.close().catch(() => {
    process.stderr.write("Deterministic native fixture MCP cleanup failed.\n");
    process.exitCode = 3;
  });
});
