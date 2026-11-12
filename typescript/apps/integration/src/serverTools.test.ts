import assert from "node:assert/strict";
import test from "node:test";
import { credentials, Metadata } from "@grpc/grpc-js";
import { AuthenticationClient } from "@chapchoo/rpc-client/accounts";
import {
  ConversationsClient,
  SetupFailure,
} from "@chapchoo/rpc-client/conversations";
import {
  Action,
  AgentsClient,
  ComputersClient,
  Power,
} from "@chapchoo/rpc-client/computers";
import { Assembly } from "@chapchoo/testenv/assembly";
import { nativeConversation } from "@chapchoo/testenv/nativeConversation";
import { lifecycleReady, poll } from "./readiness.ts";
import type {
  CallOptions,
  ClientUnaryCall,
  requestCallback,
} from "@grpc/grpc-js";
import type { Agent } from "@chapchoo/rpc-client/computers";
import type {
  HarnessSupply,
  OwnerConversation,
} from "@chapchoo/rpc-client/conversations";

type Unary<I, O> = (
  input: I,
  metadata: Metadata,
  options: CallOptions,
  callback: requestCallback<O>,
) => ClientUnaryCall;

interface Scenario {
  name: string;
  initialSupply: HarnessSupply;
  resumePath: string;
  expectedResume: "ready" | "rejected";
}
const scenarios: Scenario[] = [
  {
    name: "real native initializes MCP and preserves rejected empty restoration without model work",
    initialSupply: { managed: {} },
    resumePath: "/opt/chapchoo/native-tools-codex",
    expectedResume: "rejected",
  },
  {
    name: "controlled native fixture establishes MCP on new and exact-resumed native owners",
    initialSupply: { existingCodexPath: "/opt/chapchoo/codex-fixture" },
    resumePath: "/opt/chapchoo/codex-fixture",
    expectedResume: "ready",
  },
];
for (const scenario of scenarios) {
  void test(scenario.name, { timeout: 600_000 }, async (context) => {
    const assembly = await Assembly.createDockerIntegration();
    const endpoint = assembly.generalRpcEndpoint;
    const options = { "grpc.enable_retries": 0 };
    const auth = new AuthenticationClient(
      endpoint,
      credentials.createInsecure(),
      options,
    );
    const agents = new AgentsClient(
      endpoint,
      credentials.createInsecure(),
      options,
    );
    const computers = new ComputersClient(
      endpoint,
      credentials.createInsecure(),
      options,
    );
    const conversations = new ConversationsClient(
      endpoint,
      credentials.createInsecure(),
      options,
    );
    let fixture: Awaited<ReturnType<typeof nativeConversation>> | null = null;
    let expectedRequestId = "conversation-create";
    context.after(async () => {
      const failures: unknown[] = [];
      try {
        await fixture?.close(expectedRequestId);
      } catch (error) {
        failures.push(error);
      }
      auth.close();
      agents.close();
      computers.close();
      conversations.close();
      try {
        await assembly.close();
      } catch (error) {
        failures.push(error);
      }
      if (failures.length)
        throw new AggregateError(failures, "Native tools cleanup failed");
    });
    let token: string | null = null;
    const call = <I, O>(method: Unary<I, O>, input: I): Promise<O> =>
      new Promise((resolve, reject) => {
        const metadata = new Metadata();
        if (token) metadata.set("authorization", `Bearer ${token}`);
        method(
          input,
          metadata,
          { deadline: Date.now() + 15_000 },
          (error, result) => {
            if (error || result === undefined) {
              reject(
                new Error(
                  `Native tools RPC failed (${error?.code ?? "missing"}); response withheld`,
                ),
              );
            } else resolve(result);
          },
        );
      });
    fixture = await nativeConversation(assembly, scenario.initialSupply);
    const native = fixture;
    token = (await call(auth.passwordLogin.bind(auth), assembly.administrator))
      .token;
    const ready = (value: OwnerConversation): boolean => {
      if (value.startup?.failed !== undefined)
        throw new Error("Real native server-tools setup was rejected");
      return value.startup?.sessionReady !== undefined;
    };
    const original = await poll(native.current, ready, context.signal, 240_000);
    const target = original.startup?.sessionReady;
    assert.ok(target, "New native startup must bind an exact session");
    assert.equal(await native.pendingInputs(), 0);
    assert.equal(
      original.startup?.requestedSupply?.existingCodexPath,
      scenario.initialSupply.existingCodexPath,
    );
    assert.equal(
      original.startup?.requestedSupply?.managed !== undefined,
      scenario.initialSupply.managed !== undefined,
    );
    const originalOrdinal = original.conversation?.position?.ordinal;
    // Startup has no captured state until native emits a real event. A changed
    // saved supply distinguishes the successor's durable setup from the old
    // retained Ready fact without synthesizing a native event or polling a race.
    const existingCodexPath = scenario.resumePath;
    await call(conversations.saveSupply.bind(conversations), {
      agentId: native.agentId,
      requestId: "tools-existing-supply",
      expectedRequestId: "conversation-fixture-supply",
      supply: { existingCodexPath },
    });
    const unchangedSupply = (await native.current()).startup?.requestedSupply;
    assert.equal(
      unchangedSupply?.existingCodexPath,
      scenario.initialSupply.existingCodexPath,
    );
    assert.equal(
      unchangedSupply?.managed !== undefined,
      scenario.initialSupply.managed !== undefined,
    );
    await call(computers.requestLifecycle.bind(computers), {
      agentId: native.agentId,
      requestId: "tools-stop",
      expectedRequestId,
      action: Action.ACTION_STOP,
    });
    expectedRequestId = "tools-stop";
    const get = (): Promise<Agent> =>
      call(agents.get.bind(agents), { agentId: native.agentId });
    await poll(
      get,
      (value) =>
        lifecycleReady(
          value.computer,
          expectedRequestId,
          Power.POWER_STOPPED,
          false,
        ),
      context.signal,
    );
    await call(computers.requestLifecycle.bind(computers), {
      agentId: native.agentId,
      requestId: "tools-start",
      expectedRequestId,
      action: Action.ACTION_START,
    });
    expectedRequestId = "tools-start";
    await poll(
      get,
      (value) =>
        lifecycleReady(
          value.computer,
          expectedRequestId,
          Power.POWER_RUNNING,
          true,
        ),
      context.signal,
    );
    const restored = await poll(
      native.current,
      (value) => {
        if (
          value.startup?.requestedSupply?.existingCodexPath !==
          existingCodexPath
        )
          return false;
        if (scenario.expectedResume === "rejected")
          return value.startup.failed !== undefined;
        return (
          ready(value) &&
          value.conversation?.position !== undefined &&
          (originalOrdinal === undefined ||
            value.conversation.position.ordinal > originalOrdinal)
        );
      },
      context.signal,
    );
    if (scenario.expectedResume === "rejected") {
      assert.equal(
        restored.startup?.failed,
        SetupFailure.SETUP_FAILURE_NATIVE_REJECTED,
      );
    } else {
      assert.deepEqual(restored.startup?.sessionReady, target);
      assert.ok(restored.conversation?.position);
    }
    assert.equal(restored.conversation?.snapshotId, target.snapshotId);
    assert.equal(
      restored.conversation?.nativeSessionId,
      target.nativeSessionId,
    );
    assert.equal(await native.pendingInputs(), 0);
    console.info(
      `Native MCP acceptance: ${scenario.expectedResume}; exact target preserved; no input or model turn admitted`,
    );
  });
}
