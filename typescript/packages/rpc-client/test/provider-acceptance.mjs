// Real disposable-cluster/Standalone acceptance through generated product contracts.
// Arguments contain only endpoints, identities and private file paths, never credentials.
import { readFile } from "node:fs/promises";
import process from "node:process";
import console from "node:console";
import { randomBytes } from "node:crypto";
import { Buffer } from "node:buffer";
import { AuthenticationsClient } from "../dist/generated/authentication.js";
import { setTimeout as delay } from "node:timers/promises";
import { credentials, Metadata, status } from "@grpc/grpc-js";
import {
  InstallationClient,
  TeamManagementClient,
} from "../dist/generated/accounts.js";
import {
  AgentsClient,
  ComputersClient,
  ProviderCatalogClient,
  Action,
  Power,
  Connectivity,
  Phase,
} from "../dist/generated/computers.js";
import {
  ProviderAuthenticationClient,
  ProviderKind,
} from "../dist/generated/control.js";
const [endpoint, registrationKey, proofFile, saFile, wrongAudienceFile] =
  process.argv.slice(2);
if (
  !/^(127\.0\.0\.1|localhost):\d+$/.test(endpoint ?? "") ||
  !registrationKey ||
  !proofFile
) {
  throw new Error("Explicit disposable acceptance inputs required");
}
const options = { "grpc.enable_retries": 0 };
const clients = [];
function client(Type) {
  const value = new Type(endpoint, credentials.createInsecure(), options);
  clients.push(value);
  return value;
}
function call(instance, method, value, token) {
  const metadata = new Metadata();
  if (token) {
    metadata.set("authorization", `Bearer ${token}`);
  }
  return new Promise((resolve, reject) =>
    instance[method](
      value,
      metadata,
      { deadline: Date.now() + 15000 },
      (error, result) => {
        if (error) {
          reject(
            Object.assign(
              new Error(`Acceptance ${method} failed; response withheld`),
              { code: error.code },
            ),
          );
        } else {
          resolve(result);
        }
      },
    ),
  );
}
function check(value, message) {
  if (!value) {
    throw new Error(message);
  }
}
async function poll(read, ready) {
  const deadline = Date.now() + 120000;
  while (Date.now() < deadline) {
    const value = await read();
    if (ready(value)) {
      return value;
    }
    await delay(200);
  }
  throw new Error("Acceptance observation deadline exceeded");
}
async function denied(operation) {
  try {
    await operation();
  } catch (error) {
    check(
      [status.PERMISSION_DENIED, status.UNAUTHENTICATED].includes(error.code),
      "Wrong rejection class",
    );
    return;
  }
  throw new Error("Forbidden authentication was accepted");
}
try {
  const auth = client(ProviderAuthenticationClient);
  const configuration = {
    registrationKey,
    name: "Acceptance",
    kind: ProviderKind.PROVIDER_KIND_KUBERNETES,
    capabilities: { create: true, start: true, stop: true, restart: true },
  };
  await denied(() =>
    call(auth, "exchange", { configuration, requestId: "network-trust" }),
  );
  if (saFile) {
    const serviceAccountToken = (await readFile(saFile, "utf8")).trim();
    const exchangeId = `${Date.now()}:reviewed-acceptance`;
    const good = await call(auth, "exchange", {
      configuration,
      requestId: exchangeId,
      serviceAccountToken,
    });
    check(
      good.key?.issuedAt && good.key?.expiresAt && good.key?.token,
      "Provider credential envelope absent",
    );
    const expiry =
      Number(good.key.expiresAt.seconds) - Number(good.key.issuedAt.seconds);
    check(expiry === 900, "Provider key validity is not fifteen minutes");
    const recovered = await call(auth, "exchange", {
      configuration,
      requestId: exchangeId,
      serviceAccountToken,
    });
    check(
      recovered.key?.token === good.key.token &&
        recovered.providerId === good.providerId,
      "Exchange recovery changed credential",
    );
    await denied(() =>
      call(auth, "exchange", {
        configuration: {
          ...configuration,
          registrationKey: "unpermitted-provider",
        },
        requestId: "foreign-id",
        serviceAccountToken,
      }),
    );
    if (wrongAudienceFile) {
      const wrongAudience = (await readFile(wrongAudienceFile, "utf8")).trim();
      await denied(() =>
        call(auth, "exchange", {
          configuration,
          requestId: "wrong-audience",
          serviceAccountToken: wrongAudience,
        }),
      );
    }
  }
  const installation = client(InstallationClient);
  const teams = client(TeamManagementClient);
  const agents = client(AgentsClient);
  const computers = client(ComputersClient);
  const catalog = client(ProviderCatalogClient);
  const authentications = client(AuthenticationsClient);
  const email = `acceptance-${randomBytes(8).toString("hex")}@example.invalid`;
  const session = await call(installation, "initialize", {
    installationProof: (await readFile(proofFile, "utf8")).trim(),
    email,
    password: `Aa1!${randomBytes(24).toString("base64url")}`,
  });
  const team = (await call(teams, "listTeams", {}, session.token)).items[0];
  check(team, "Personal team absent");
  const providers = await poll(
    () => call(catalog, "list", { limit: 20 }, session.token),
    (result) => result.items.length > 0,
  );
  check(
    providers.items.length === 1,
    "Isolated fixture must have one Provider",
  );
  const providerId = providers.items[0]?.id;
  check(
    providerId && providerId !== registrationKey,
    "Canonical Provider identity absent",
  );
  const image = (
    await call(agents, "listProfileImages", { limit: 20 }, session.token)
  ).items[0];
  check(image, "Profile catalogue item absent");
  const key = Buffer.from("disposable-acceptance-key");
  let authenticationId;
  try {
    const registered = await call(
      authentications,
      "registerKey",
      {
        scope: { personal: {} },
        requestId: "acceptance-key",
        name: "Acceptance only",
        key,
      },
      session.token,
    );
    authenticationId = registered.record?.authenticationId;
    check(authenticationId, "Explicit authentication is required");
  } finally {
    key.fill(0);
  }
  const created = await call(
    agents,
    "create",
    {
      requestId: "create",
      providerId,
      authenticationId,
      name: "Authentication acceptance",
      teamId: team.id,
      handle: "auth-acceptance",
      profileImageId: image.id,
      paletteKey: "sage",
    },
    session.token,
  );
  check(created.agent, "Agent not allocated");
  const agentId = created.agent.id;
  const read = () => call(agents, "get", { agentId }, session.token);
  const ready = (value, power) => {
    check(
      value.computer?.phase !== Phase.PHASE_BLOCKED,
      "Lifecycle blocked; mutation not replayed",
    );
    return (
      value.computer?.power === power &&
      value.computer?.phase === Phase.PHASE_COMPLETED
    );
  };
  await poll(
    read,
    (value) =>
      ready(value, Power.POWER_RUNNING) &&
      value.computer?.connector === Connectivity.CONNECTIVITY_CONNECTED,
  );
  console.info(
    "Provider acceptance: authenticated, created and Connector connected",
  );
  await call(
    computers,
    "requestLifecycle",
    {
      agentId,
      requestId: "stop",
      expectedRequestId: "create",
      action: Action.ACTION_STOP,
    },
    session.token,
  );
  await poll(read, (value) => ready(value, Power.POWER_STOPPED));
  await call(
    computers,
    "requestLifecycle",
    {
      agentId,
      requestId: "start",
      expectedRequestId: "stop",
      action: Action.ACTION_START,
    },
    session.token,
  );
  await poll(
    read,
    (value) =>
      ready(value, Power.POWER_RUNNING) &&
      value.computer?.connector === Connectivity.CONNECTIVITY_CONNECTED,
  );
  await call(
    computers,
    "requestLifecycle",
    {
      agentId,
      requestId: "delete",
      expectedRequestId: "start",
      action: Action.ACTION_DELETE,
    },
    session.token,
  );
  await poll(async () => {
    try {
      await read();
      return false;
    } catch (error) {
      if (error.code === status.NOT_FOUND) {
        return true;
      }
      throw error;
    }
  }, Boolean);
  console.info(
    "Provider acceptance: stop/start/delete completed; no secret output",
  );
} finally {
  for (const value of clients) {
    value.close();
  }
}
