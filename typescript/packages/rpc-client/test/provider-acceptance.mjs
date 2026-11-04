// Real disposable-cluster/Standalone acceptance through generated product contracts.
// Arguments contain only endpoints, identities and private file paths, never credentials.
import { readFile } from "node:fs/promises";
import process from "node:process";
import console from "node:console";
import { randomBytes } from "node:crypto";
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
const [endpoint, providerId, proofFile, saFile, wrongAudienceFile] =
  process.argv.slice(2);
if (
  !/^(127\.0\.0\.1|localhost):\d+$/.test(endpoint ?? "") ||
  !providerId ||
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
    providerId,
    name: "Acceptance",
    kind: ProviderKind.PROVIDER_KIND_KUBERNETES,
    capabilities: { create: true, start: true, stop: true, restart: true },
  };
  await denied(() =>
    call(auth, "exchange", { configuration, requestId: "network-trust" }),
  );
  if (saFile) {
    const serviceAccountToken = (await readFile(saFile, "utf8")).trim();
    const good = await call(auth, "exchange", {
      configuration,
      requestId: "reviewed-acceptance",
      serviceAccountToken,
    });
    const expiry =
      Number(good.expiresAt.seconds) - Number(good.issuedAt.seconds);
    check(expiry === 900, "Provider key validity is not fifteen minutes");
    const recovered = await call(auth, "exchange", {
      configuration,
      requestId: "reviewed-acceptance",
      serviceAccountToken,
    });
    check(
      recovered.token === good.token,
      "Exchange recovery changed credential",
    );
    await denied(() =>
      call(auth, "exchange", {
        configuration: { ...configuration, providerId: "unpermitted-provider" },
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
  const email = `acceptance-${randomBytes(8).toString("hex")}@example.invalid`;
  const session = await call(installation, "initialize", {
    installationProof: (await readFile(proofFile, "utf8")).trim(),
    email,
    password: `Aa1!${randomBytes(24).toString("base64url")}`,
  });
  const team = (await call(teams, "listTeams", {}, session.token)).items[0];
  check(team, "Personal team absent");
  await poll(
    () => call(catalog, "list", { limit: 20 }, session.token),
    (result) => result.items.some((item) => item.id === providerId),
  );
  const created = await call(
    agents,
    "create",
    {
      requestId: "create",
      providerId,
      name: "Authentication acceptance",
      teamId: team.id,
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
