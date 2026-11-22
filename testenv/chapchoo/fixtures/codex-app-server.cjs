#!/usr/bin/env node
// Deterministic native-protocol fixture. This executable never contacts a model.
// Select it explicitly as an existing CLI in an isolated test Computer only.
const { createInterface } = require("node:readline");
const { existsSync, readFileSync, writeFileSync, watch } = require("node:fs");
const { join } = require("node:path");
const { execFile } = require("node:child_process");
const { promisify } = require("node:util");
const runFile = promisify(execFile);

/** Optional test-owned barrier: positive native work remains live until release. */
async function awaitWorkRelease() {
  const path = "/workspace/acceptance-work-hold";
  if (!existsSync(path)) return;
  await new Promise((resolve, reject) => {
    const watcher = watch(
      "/workspace",
      {
        signal: AbortSignal.timeout(180_000),
      },
      () => {
        if (!existsSync(path)) watcher.close();
      },
    );
    watcher.once("error", reject);
    watcher.once("close", () => {
      if (existsSync(path))
        reject(new Error("Fixture working gate closed without release"));
      else resolve();
    });
    // Recheck after synchronous watch registration so release cannot be missed.
    if (!existsSync(path)) watcher.close();
  });
}
if (process.argv.includes("--version")) {
  process.stdout.write("codex-cli 0.171.0\n");
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
  if (catalog.tools.length !== 1 || catalog.tools[0]?.name !== "send")
    throw new Error("Fixture requires its exact server-tools catalog");
  mcpTools = catalog.tools;
}
let threadId = "fixture-thread";
let turnId = "fixture-turn";
const model = "fixture-model";
const alternateModel = "fixture-alternate";
let authenticated = false;
let directClient = false;
let usesReplacementKey = false;
let active = false;
let primaryUsed = false;
let primaryStarted = false;
const helperThreads = new Set();
let cwd = "/workspace";
// These fixture-only literals contain no request, response or credential data.
async function publishFixture(which, publication) {
  if (
    !["first", "refresh", "question", "result", "implicit"].includes(which) ||
    !mcp
  )
    throw new Error("Fixture publication owner is unavailable");
  const marker = `/workspace/acceptance-publication-${which}`;
  const stage = (value) => writeFileSync(marker, `${value}\n`, { mode: 0o600 });
  stage("mcp-call-started");
  let result;
  try {
    result = await mcp.callTool({
      name: publication.tool,
      arguments: publication.arguments,
    });
  } catch (error) {
    stage("mcp-call-exception");
    throw error;
  }
  stage(result.isError ? "mcp-result-error" : "mcp-result-accepted");
  if (result.isError) throw new Error("Fixture publication was not accepted");
  return result;
}
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
  cliVersion: "0.171.0",
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
function verifyWorkInstructions(workspace) {
  const instructions = readFileSync(join(workspace, "AGENTS.md"), "utf8");
  if (
    !instructions.includes("Use send to publish a message") ||
    !instructions.includes("request_input") ||
    instructions.includes("send_message")
  )
    throw new Error(
      "Fixture requires the revised Conversation Work instructions",
    );
  return instructions;
}
async function workTurn(id, params, which, args = null) {
  if (!authenticated || active || params.threadId !== threadId)
    throw new Error("Fixture work turn was not admitted");
  writeFileSync(
    join(process.env.CODEX_HOME, `${which}-work-turn-once`),
    "accepted\n",
    {
      flag: "wx",
      mode: 0o600,
    },
  );
  active = true;
  turnId = `fixture-${which}-turn`;
  reply(id, { turn: turn("inProgress") });
  notify("turn/started", { threadId, turn: turn("inProgress") });
  // Native records include both system continuation and ordinary input.
  // Only ordinary input creates a User Chat receipt.
  const received = {
    type: "userMessage",
    id: `native-${which}-input`,
    clientId: params.clientUserMessageId ?? null,
    content: params.input,
  };
  item("item/started", received);
  item("item/completed", received);
  if (!args) {
    await awaitWorkRelease();
    active = false;
    notify("turn/completed", { threadId, turn: turn("completed") });
    notify("thread/status/changed", { threadId, status: { type: "idle" } });
    return;
  }
  const publication = {
    type: "mcpToolCall",
    id: `publication-${which}`,
    server: "chapchoo",
    tool: "send",
    arguments: args,
    status: "inProgress",
    result: null,
    error: null,
    durationMs: null,
  };
  item("item/started", publication);
  const result = await publishFixture(which, publication);
  item("item/completed", {
    ...publication,
    result,
    status: "completed",
    durationMs: 0,
  });
  active = false;
  notify("turn/completed", { threadId, turn: turn("completed") });
  notify("thread/status/changed", { threadId, status: { type: "idle" } });
}
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
      directClient = params?.clientInfo?.name === "chapchoo";
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
      usesReplacementKey =
        params.apiKey === "disposable-replaced-conversation-key";
      if (params.apiKey === "disposable-fresh-conversation-key") {
        threadId = "fixture-fresh-thread";
      }
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
      reply(id, {
        data: [
          { id: ":workspace", allowed: true },
          { id: ":danger-full-access", allowed: true },
        ],
        nextCursor: null,
      });
      break;
    case "configRequirements/read":
      reply(id, { requirements: null });
      break;
    case "experimentalFeature/list":
      reply(id, {
        data: [{ name: "guardian_approval", enabled: true }],
        nextCursor: null,
      });
      break;
    case "model/list":
      reply(id, {
        data: [model, alternateModel].map((value) => ({
          id: value,
          model: value,
          displayName:
            value === model
              ? "Deterministic fixture"
              : "Alternate fixture model",
          description: "No model request is made",
          isDefault: value === model,
          supportedReasoningEfforts: [
            { reasoningEffort: "medium", description: "Fixture default" },
            ...(value === alternateModel
              ? [{ reasoningEffort: "high", description: "Fixture high" }]
              : []),
          ],
          defaultReasoningEffort: "medium",
          serviceTiers:
            value === alternateModel
              ? [{ id: "priority", name: "Fast", description: "Fixture fast" }]
              : [],
          defaultServiceTier: null,
          inputModalities: ["text"],
          supportsPersonality: false,
          supportsParallelToolCalls: false,
        })),
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
          (params.threadId !== threadId ||
            !existsSync(retainedThread) ||
            readFileSync(retainedThread, "utf8").trim() !== threadId)
        )
          throw new Error("Fixture exact retained thread is unavailable");
        primaryStarted = true;
        await connectMcp(params.config?.mcp_servers);
        if (method === "thread/start")
          writeFileSync(retainedThread, `${threadId}\n`, {
            flag: "wx",
            mode: 0o600,
          });
        if (directClient) {
          writeFileSync(
            "/workspace/acceptance-direct-admission",
            method === "thread/start" ? "start\n" : "resume\n",
          );
          if (usesReplacementKey) {
            writeFileSync(
              "/workspace/acceptance-direct-credential",
              "replacement\n",
            );
          }
        }
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
      const prompt = params?.input?.[0]?.text;
      if (
        prompt ===
          "Exercise the deterministic native conversation with a selected model" &&
        (params.model !== alternateModel ||
          params.effort !== "high" ||
          params.serviceTierForTurn !== "priority" ||
          params.permissions !== ":workspace" ||
          params.approvalPolicy !== "on-request" ||
          params.approvalsReviewer !== "user")
      ) {
        throw new Error("The saved preferences did not reach the native turn");
      }
      if (
        turnId === "fixture-unanswered-turn" ||
        turnId === "fixture-refresh-turn"
      ) {
        const header = "[System-generated input]\nkind: continuation\n\n";
        if (
          !prompt?.startsWith(header) ||
          params.input.length !== 1 ||
          params.input[0].type !== "text"
        ) {
          throw new Error(
            "Fixture continuation did not use the system input format",
          );
        }
        const body = prompt.slice(header.length);
        const context = JSON.parse(body.slice(body.indexOf("\n") + 1));
        const [work] = context;
        if (turnId === "fixture-unanswered-turn") {
          if (
            context.length !== 1 ||
            work.disposition !== "active" ||
            work.objective !==
              "Reply to the user's messages and explicitly settle the Work." ||
            work.title !== undefined ||
            work.tasks !== undefined
          )
            throw new Error(
              "Fixture continuation did not use hidden reply work",
            );
          await workTurn(id, params, "implicit", {
            action: "finish",
            message: "Fixture replied without a work plan.",
          });
          break;
        }
        if (
          context.length !== 1 ||
          work.title !== "Check revised conversation fixture" ||
          work.disposition !== "active" ||
          JSON.stringify(work.tasks) !==
            JSON.stringify([
              { text: "Choose a channel", status: "in_progress" },
              { text: "Check saved instructions", status: "completed" },
            ])
        )
          throw new Error("Fixture continuation did not use amended work");
        await workTurn(id, params, "question", {
          action: "request_input",
          message: "Which fixture channel should I use?",
        });
        break;
      }
      if (prompt === "Use the stable fixture channel") {
        if (turnId !== "fixture-question-turn")
          throw new Error("Fixture reply came before its question");
        await workTurn(id, params, "result", {
          action: "finish",
          message: "Fixture work finished on stable.",
        });
        break;
      }
      if (
        prompt ===
        "[System-generated input]\nExercise a reply without a work plan"
      ) {
        await workTurn(id, params, "unanswered");
        break;
      }
      if (prompt === "Continue after refreshing project instructions") {
        if (
          !authenticated ||
          active ||
          primaryUsed ||
          params.threadId !== threadId ||
          !verifyWorkInstructions(cwd).includes("PROJECT_BETA")
        )
          throw new Error("Fixture refresh input was not admitted");
        // The exclusive durable marker catches duplicate forwarding across
        // physical fixture restarts without silently replaying the input.
        writeFileSync(
          join(process.env.CODEX_HOME, "refresh-input-once"),
          "accepted\n",
          { flag: "wx", mode: 0o600 },
        );
        primaryUsed = true;
        active = true;
        turnId = "fixture-refresh-turn";
        reply(id, { turn: turn("inProgress") });
        notify("turn/started", { threadId, turn: turn("inProgress") });
        item("item/started", {
          type: "userMessage",
          id: "native-refresh-user",
          clientId: params.clientUserMessageId ?? null,
          content: params.input,
        });
        item("item/completed", {
          type: "userMessage",
          id: "native-refresh-user",
          clientId: params.clientUserMessageId ?? null,
          content: params.input,
        });
        const refreshedPublication = {
          type: "mcpToolCall",
          id: "publication-refresh",
          server: "chapchoo",
          tool: "send",
          arguments: {
            action: "continue",
            title: "Check revised conversation fixture",
            tasks: [
              { text: "Choose a channel", status: "in_progress" },
              { text: "Check saved instructions", status: "completed" },
            ],
            message: "Refreshed project fixture publication.",
          },
          status: "inProgress",
          result: null,
          error: null,
          durationMs: null,
        };
        item("item/started", refreshedPublication);
        const refreshedResult = await publishFixture(
          "refresh",
          refreshedPublication,
        );
        item("item/completed", {
          ...refreshedPublication,
          status: "completed",
          result: refreshedResult,
          durationMs: 0,
        });
        // The real Codex uses the managed SessionStart hook for this call.
        // This fixture proves the same shipped script and product lookup end to end.
        const { stdout } = await runFile(
          process.env.CHAPCHOO_GOAL_RESTORE_NODE,
          [process.env.CHAPCHOO_GOAL_RESTORE_SCRIPT],
          { timeout: 2500 },
        );
        const restored = JSON.parse(stdout).hookSpecificOutput;
        const [current] = JSON.parse(
          restored.additionalContext.split("\n").at(-1),
        );
        if (
          restored.hookEventName !== "SessionStart" ||
          current.title !== refreshedPublication.arguments.title ||
          current.disposition !== "active" ||
          JSON.stringify(current.tasks) !==
            JSON.stringify(refreshedPublication.arguments.tasks)
        )
          throw new Error("Fixture restore did not read the current Work");
        active = false;
        notify("turn/completed", { threadId, turn: turn("completed") });
        notify("thread/status/changed", { threadId, status: { type: "idle" } });
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
        (params.input[0]?.text !==
          "Exercise the deterministic native conversation" &&
          prompt !==
            "Exercise the deterministic native conversation with a selected model" &&
          !(
            threadId === "fixture-fresh-thread" &&
            prompt === "Send the first message to the new conversation"
          ))
      )
        throw new Error("Fixture turn was not admitted");
      verifyWorkInstructions(cwd);
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
        tool: "send",
        arguments: {
          action: "continue",
          title: "Review conversation fixture",
          tasks: [
            { text: "Check saved instructions", status: "in_progress" },
            { text: "Choose a channel", status: "pending" },
          ],
          message: "Intentional fixture publication.",
        },
        status: "inProgress",
        result: null,
        error: null,
        durationMs: null,
      };
      item("item/started", publication);
      const publicationStart = performance.now();
      const result = await publishFixture("first", publication);
      item("item/completed", {
        ...publication,
        status: "completed",
        result,
        durationMs: Math.round(performance.now() - publicationStart),
      });
      await awaitWorkRelease();
      write({
        id: "fixture-form",
        method: "mcpServer/elicitation/request",
        params: {
          threadId,
          turnId,
          serverName: "chapchoo",
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
