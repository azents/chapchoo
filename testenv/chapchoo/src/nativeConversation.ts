import { setTimeout } from "node:timers/promises";
import { credentials, Metadata } from "@grpc/grpc-js";
import {
  AuthenticationClient,
  TeamManagementClient,
} from "@chapchoo/rpc-client/accounts";
import { AuthenticationsClient } from "@chapchoo/rpc-client/authentication";
import { ConversationsClient } from "@chapchoo/rpc-client/conversations";
import type { OwnerConversation } from "@chapchoo/rpc-client/conversations";
import {
  AgentsClient,
  ComputersClient,
  ProviderCatalogClient,
  Action,
  Power,
  Connectivity,
  Phase,
} from "@chapchoo/rpc-client/computers";
import type {
  CallOptions,
  ClientUnaryCall,
  requestCallback,
} from "@grpc/grpc-js";
import type { Assembly } from "./assembly.js";

type Unary<I, O> = (
  input: I,
  metadata: Metadata,
  options: CallOptions,
  callback: requestCallback<O>,
) => ClientUnaryCall;
async function poll<T>(
  read: () => Promise<T>,
  ready: (value: T) => boolean,
): Promise<T> {
  const signal = AbortSignal.timeout(180_000);
  while (true) {
    signal.throwIfAborted();
    const value = await read();
    if (ready(value)) return value;
    await setTimeout(100, undefined, { signal });
  }
}
/** Disposable prerequisite state uses the same public RPC admission as the UI. */
export async function nativeConversation(assembly: Assembly): Promise<{
  chatPath: string;
  current: () => Promise<OwnerConversation>;
  close: () => Promise<void>;
}> {
  const docker = assembly.docker;
  if (!docker)
    throw new Error("Conversation acceptance requires an owned Docker fixture");
  const endpoint = assembly.generalRpcEndpoint;
  const options = { "grpc.enable_retries": 0 };
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
  const keys = new AuthenticationsClient(
    endpoint,
    credentials.createInsecure(),
    options,
  );
  const conversations = new ConversationsClient(
    endpoint,
    credentials.createInsecure(),
    options,
  );
  const clients = [
    auth,
    teams,
    agents,
    computers,
    catalog,
    keys,
    conversations,
  ];
  let token: string | null = null;
  let agentId: string | null = null;
  let stage = "initialize";
  const call = <I, O>(method: Unary<I, O>, input: I): Promise<O> =>
    new Promise((resolve, reject) => {
      const metadata = new Metadata();
      if (token) metadata.set("authorization", `Bearer ${token}`);
      method(
        input,
        metadata,
        { deadline: Date.now() + 15_000 },
        (error, result) => {
          if (error || result === undefined)
            reject(
              new Error(
                `Conversation fixture ${stage} RPC failed (${error?.code ?? "missing"}); response withheld`,
              ),
            );
          else resolve(result);
        },
      );
    });
  const close = async (): Promise<void> => {
    try {
      if (agentId) {
        await call(computers.requestLifecycle.bind(computers), {
          agentId,
          requestId: "conversation-delete",
          expectedRequestId: "conversation-create",
          action: Action.ACTION_DELETE,
        });
        await poll(() => docker.productAbsent(), Boolean);
      }
    } finally {
      for (const client of clients) client.close();
    }
  };
  try {
    await assembly.initializeAdministrator();
    stage = "login";
    token = (await call(auth.passwordLogin.bind(auth), assembly.administrator))
      .token;
    stage = "team";
    const team = (await call(teams.listTeams.bind(teams), {})).items[0];
    if (!team) throw new Error("Conversation fixture requires a personal Team");
    const provider = await assembly.startDockerProvider();
    stage = "provider";
    const available = await poll(
      () => call(catalog.list.bind(catalog), { limit: 20 }),
      (value) => value.items.length === 1,
    );
    const providerId = available.items[0]?.id;
    if (!providerId)
      throw new Error("Conversation fixture requires its registered Provider");
    docker.bindProvider(providerId);
    stage = "image";
    const image = (
      await call(agents.listProfileImages.bind(agents), { limit: 20 })
    ).items[0];
    if (!image)
      throw new Error("Conversation fixture requires a catalogue image");
    const key = Buffer.from("disposable-native-conversation-key");
    let authenticationId: string | undefined;
    stage = "key";
    try {
      authenticationId = (
        await call(keys.registerKey.bind(keys), {
          scope: { personal: {} },
          requestId: "conversation-key",
          name: "Deterministic fixture only",
          key,
        })
      ).record?.authenticationId;
    } finally {
      key.fill(0);
    }
    if (!authenticationId)
      throw new Error("Conversation fixture key registration was not observed");
    stage = "create";
    // Freeze only this owned Provider before creating work. The explicit
    // deterministic executable must be selected before any Connector can start.
    const agent = await provider.whileSuspended(async () => {
      const agent = (
        await call(agents.create.bind(agents), {
          requestId: "conversation-create",
          providerId,
          authenticationId,
          teamId: team.id,
          handle: "chat-agent",
          name: "Conversation fixture",
          profileImageId: image.id,
          paletteKey: "sage",
        })
      ).agent;
      if (!agent)
        throw new Error("Conversation fixture Agent was not returned");
      agentId = agent.id;
      stage = "select-deterministic-executable";
      await call(conversations.saveSupply.bind(conversations), {
        agentId,
        requestId: "conversation-fixture-supply",
        supply: { existingCodexPath: "/opt/chapchoo/codex-fixture" },
      });
      return agent;
    });
    stage = "connect";
    await poll(
      () => call(agents.get.bind(agents), { agentId: agent.id }),
      (value) => {
        if (value.computer?.phase === Phase.PHASE_BLOCKED) {
          throw new Error(
            "Conversation fixture native allocation is blocked; no mutation retry",
          );
        }
        return (
          value.computer?.power === Power.POWER_RUNNING &&
          value.computer.connector === Connectivity.CONNECTIVITY_CONNECTED &&
          value.computer.phase === Phase.PHASE_COMPLETED &&
          value.computer.requestId === "conversation-create"
        );
      },
    );
    return {
      chatPath: `/teams/my/agents/${agent.handle}/chat`,
      current: () =>
        call(conversations.current.bind(conversations), { agentId: agent.id }),
      close,
    };
  } catch (error) {
    try {
      await close();
    } catch (cleanup) {
      throw new AggregateError(
        [error, cleanup],
        "Conversation fixture failed and cleanup was unconfirmed",
      );
    }
    throw error;
  }
}
