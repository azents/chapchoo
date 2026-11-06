import assert from "node:assert/strict";
import test from "node:test";
import { Buffer } from "node:buffer";
import { AuthenticationsClient } from "@chapchoo/rpc-client/authentication";
import { Assembly } from "@chapchoo/testenv/assembly";
import { credentials, Metadata, status } from "@grpc/grpc-js";
import {
  AuthenticationClient,
  TeamManagementClient,
} from "@chapchoo/rpc-client/accounts";
import {
  AgentsClient,
  ComputersClient,
  ProviderCatalogClient,
  Action,
  Power,
} from "@chapchoo/rpc-client/computers";
import { nativeEvidence } from "./nativeEvidence.ts";
import { lifecycleReady, poll } from "./readiness.ts";
import type { NativeEvidence } from "./nativeEvidence.ts";
import type { Agent } from "@chapchoo/rpc-client/computers";
import type {
  CallOptions,
  ClientUnaryCall,
  requestCallback,
} from "@grpc/grpc-js";

type Unary<I, O> = (
  input: I,
  metadata: Metadata,
  options: CallOptions,
  callback: requestCallback<O>,
) => ClientUnaryCall;
class RpcFailure extends Error {
  readonly code: status;
  constructor(code: status) {
    super(`Integration RPC failed with status ${code}; response withheld`);
    this.code = code;
  }
}
function rpcCall<I, O>(
  method: Unary<I, O>,
  input: I,
  token: string | null = null,
  signal?: AbortSignal,
): Promise<O> {
  signal?.throwIfAborted();
  return new Promise((resolve, reject) => {
    const metadata = new Metadata();
    if (token) {
      metadata.set("authorization", `Bearer ${token}`);
    }
    method(
      input,
      metadata,
      { deadline: Date.now() + 15_000 },
      (error, result) => {
        if (signal?.aborted) {
          reject(new Error("Native integration was cancelled"));
        } else if (error || result == null) {
          reject(new RpcFailure(error?.code ?? status.UNKNOWN));
        } else {
          resolve(result);
        }
      },
    );
  });
}
void test(
  "actual Control, Docker Provider and Connector preserve original data and individually clean up",
  { timeout: 420_000 },
  async (context) => {
    const controller = new AbortController();
    const signal = AbortSignal.any([context.signal, controller.signal]);
    const call = <I, O>(
      method: Unary<I, O>,
      input: I,
      token: string | null = null,
    ): Promise<O> => rpcCall(method, input, token, signal);
    const waitFor = <T>(
      read: () => Promise<T>,
      ready: (value: T) => boolean,
    ): Promise<T> => poll(read, ready, signal);
    const assembly = await Assembly.createDockerIntegration();
    context.after(() => assembly.close());
    const options = { "grpc.enable_retries": 0 };
    const endpoint = assembly.generalRpcEndpoint;
    const auth = new AuthenticationClient(
      endpoint,
      credentials.createInsecure(),
      options,
    );
    const teams = new TeamManagementClient(
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
    const catalog = new ProviderCatalogClient(
      endpoint,
      credentials.createInsecure(),
      options,
    );
    const authentications = new AuthenticationsClient(
      endpoint,
      credentials.createInsecure(),
      options,
    );
    let evidence: NativeEvidence | null = null;
    let failure: Error | null = null;
    try {
      await assembly.initializeAdministrator();
      const session = await call(
        auth.passwordLogin.bind(auth),
        assembly.administrator,
      );
      const team = (await call(teams.listTeams.bind(teams), {}, session.token))
        .items[0];
      assert.ok(team, "Personal Team is required");
      const provider = await assembly.startDockerProvider();
      const docker = assembly.docker;
      assert.ok(docker, "Real Docker fixture is required");
      const journey = async (): Promise<void> => {
        console.info("Native Docker step: register");
        const available = await waitFor(
          () => call(catalog.list.bind(catalog), { limit: 20 }, session.token),
          (choices) => choices.items.length > 0,
        );
        assert.equal(
          available.items.length,
          1,
          "Isolated fixture must have one Provider",
        );
        const providerId = available.items[0]?.id;
        assert.ok(providerId, "Canonical Provider identity is required");
        assert.notEqual(providerId, docker.registrationKey);
        docker.bindProvider(providerId);
        console.info("Native Docker step: create and connect");
        const image = (
          await call(
            agents.listProfileImages.bind(agents),
            { limit: 20 },
            session.token,
          )
        ).items[0];
        assert.ok(image, "A real catalogue image is required");
        const key = Buffer.from("disposable-native-integration-key");
        let authenticationId: string | undefined;
        try {
          const registered = await call(
            authentications.registerKey.bind(authentications),
            {
              scope: { personal: {} },
              requestId: "native-key",
              name: "Native fixture only",
              key,
            },
            session.token,
          );
          authenticationId = registered.record?.authenticationId;
        } finally {
          key.fill(0);
        }
        assert.ok(authenticationId, "Explicit authentication is required");
        const created = await call(
          agents.create.bind(agents),
          {
            requestId: "create",
            providerId,
            authenticationId,
            name: "Native integration",
            teamId: team.id,
            handle: "native-agent",
            profileImageId: image.id,
            paletteKey: "sage",
          },
          session.token,
        );
        assert.ok(created.agent, "Agent allocation must be returned");
        console.info("Native Docker step: allocation returned");
        const agentId = created.agent.id;
        const get = (): Promise<Agent> =>
          call(agents.get.bind(agents), { agentId }, session.token);
        await waitFor(get, (value) =>
          lifecycleReady(value.computer, "create", Power.POWER_RUNNING, true),
        );
        console.info("Native Docker step: connected original execution");
        const original = await docker.current();
        assert.equal(original.user, "10001:10001");
        assert.equal(original.noCopy, false);
        await docker.writeMarker(original);
        assert.equal(await docker.readMarker(original), "saved-original-data");
        assert.equal(await docker.canariesPresent(), true);
        console.info("Native Docker step: stop");
        await call(
          computers.requestLifecycle.bind(computers),
          {
            agentId,
            requestId: "stop",
            expectedRequestId: "create",
            action: Action.ACTION_STOP,
          },
          session.token,
        );
        await waitFor(get, (value) =>
          lifecycleReady(value.computer, "stop", Power.POWER_STOPPED, false),
        );
        assert.equal(await docker.containerAbsent(original.container), true);
        console.info("Native Docker step: start with original data");
        await call(
          computers.requestLifecycle.bind(computers),
          {
            agentId,
            requestId: "start",
            expectedRequestId: "stop",
            action: Action.ACTION_START,
          },
          session.token,
        );
        await waitFor(get, (value) =>
          lifecycleReady(value.computer, "start", Power.POWER_RUNNING, true),
        );
        const successor = await docker.current();
        assert.notEqual(successor.container, original.container);
        assert.equal(successor.volume, original.volume);
        assert.equal(successor.noCopy, true);
        assert.equal(await docker.readMarker(successor), "saved-original-data");
        console.info("Native Docker step: restart with empty original storage");
        await docker.clearSaved(successor);
        await call(
          computers.requestLifecycle.bind(computers),
          {
            agentId,
            requestId: "restart",
            expectedRequestId: "start",
            action: Action.ACTION_RESTART,
          },
          session.token,
        );
        await waitFor(
          () => docker.successor(successor),
          (value) => value !== null,
        );
        await waitFor(get, (value) =>
          lifecycleReady(value.computer, "restart", Power.POWER_RUNNING, true),
        );
        const restarted = await docker.current();
        assert.notEqual(restarted.container, successor.container);
        assert.equal(restarted.volume, original.volume);
        assert.equal(restarted.noCopy, true);
        assert.equal(await docker.savedEmpty(restarted), true);
        console.info("Native Docker step: delete and prove native absence");
        await call(
          computers.requestLifecycle.bind(computers),
          {
            agentId,
            requestId: "delete",
            expectedRequestId: "restart",
            action: Action.ACTION_DELETE,
          },
          session.token,
        );
        await waitFor(async () => {
          try {
            await get();
            return false;
          } catch (error) {
            if (
              error instanceof RpcFailure &&
              error.code === status.NOT_FOUND
            ) {
              return true;
            }
            throw error;
          }
        }, Boolean);
        await waitFor(() => docker.productAbsent(), Boolean);
        assert.equal(await docker.canariesPresent(), true);
        evidence = nativeEvidence(
          JSON.stringify({
            ...docker.evidence,
            original: original.container,
            successor: successor.container,
            restarted: restarted.container,
            volume: original.volume,
            cleanup: "confirmed",
            canaries: "unchanged",
            source: "explicit-unix",
            network: "bridge",
          }),
        );
        assert.ok(evidence, "Safe native evidence is required");
        await call(auth.logout.bind(auth), {}, session.token);
      };
      const runningJourney = journey();
      try {
        await Promise.race([
          runningJourney,
          provider.completion.then(async () => {
            await provider.finished(1000);
            throw new Error("Native Docker Provider exited unexpectedly");
          }),
        ]);
      } catch (error) {
        const diagnostic = provider.diagnostic();
        throw new Error(
          `Native Docker journey failed${diagnostic ? `: ${diagnostic}` : "; no complete safe Provider diagnostic"}`,
          { cause: error },
        );
      } finally {
        // Drain the losing branch before closing clients or native resources.
        // An in-flight mutation is never replayed; its finite RPC settles first.
        controller.abort(new Error("Native integration journey stopped"));
        await runningJourney.catch(() => undefined);
      }
    } catch (error) {
      failure =
        error instanceof Error ? error : new Error("Native integration failed");
    } finally {
      auth.close();
      teams.close();
      agents.close();
      computers.close();
      catalog.close();
      authentications.close();
      try {
        await assembly.close();
      } catch (cleanup) {
        if (failure) {
          throw new AggregateError(
            [failure, cleanup],
            "Native integration failed and owned cleanup was unconfirmed",
          );
        }
        throw cleanup;
      }
    }
    if (failure) {
      throw failure;
    }
    console.info(`Native Docker evidence: ${JSON.stringify(evidence)}`);
  },
);
