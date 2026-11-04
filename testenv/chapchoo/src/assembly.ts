import { access, mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  buildEnvironment,
  installedPackageRoot,
  safeEnvironment,
  dockerFixtureConfig,
  integrationBinary,
} from "./config/index.js";
import type { DockerFixtureConfig } from "./config/index.js";
import { DockerFixture } from "./docker.js";
import { ProcessOwner } from "./processes.js";
import { fixtureUser, secret, safeMessage } from "./safety.js";
import { seedRegisteredUser, seedAdministrator } from "./seed.js";
import { SyntheticKubernetes } from "./kubernetes.js";
import type { FixtureUser } from "./safety.js";
import type { OwnedChild } from "./processes.js";

const repository = fileURLToPath(new URL("../../../", import.meta.url));
const typescript = join(repository, "typescript");

async function privatePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", () =>
      reject(new Error("Could not reserve a private fixture port")),
    );
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        server.close();
        reject(new Error("No private fixture port allocated"));
        return;
      }
      const port = address.port;
      server.close((error) => {
        if (error) {
          reject(new Error("Could not release a private fixture port"));
        } else {
          resolve(port);
        }
      });
    });
  });
}

export class Assembly {
  readonly administrator: FixtureUser = fixtureUser("administrator");
  readonly member: FixtureUser = fixtureUser("member");
  readonly second: FixtureUser = fixtureUser("second");
  readonly resetPassword = `Aa1!${secret()}`;
  readonly setupProof = secret();
  private kubernetes: SyntheticKubernetes | null = null;
  readonly docker: DockerFixture | null;
  readonly teamName = "Acceptance team";
  readonly teamHandle = `acceptance-${secret().slice(0, 12).toLowerCase()}`;
  readonly generalUrl: string;
  readonly adminUrl: string;
  private readonly owner: ProcessOwner;
  private postgresAttempted = false;
  private closed = false;
  private product: OwnedChild | null = null;
  private productReady = false;

  private constructor(
    private readonly directory: string,
    private readonly generalPort: number,
    private readonly adminPort: number,
    generalWebPort: number,
    adminWebPort: number,
    private readonly packageRoot: string | null,
    docker: DockerFixtureConfig | null,
    private readonly nativeBinary: string | null,
  ) {
    this.docker = docker ? new DockerFixture(docker) : null;
    this.owner = new ProcessOwner(join(directory, "owned-processes.jsonl"));
    this.generalUrl = `http://127.0.0.1:${generalWebPort}${packageRoot ? "" : "/app"}`;
    this.adminUrl = `http://127.0.0.1:${adminWebPort}${packageRoot ? "" : "/admin"}`;
  }
  static async create(): Promise<Assembly> {
    return Assembly.createOwned(false);
  }
  static async createDockerIntegration(): Promise<Assembly> {
    return Assembly.createOwned(true);
  }
  private static async createOwned(realDocker: boolean): Promise<Assembly> {
    const docker = realDocker ? dockerFixtureConfig() : null;
    const packageRoot = realDocker ? null : installedPackageRoot();
    const nativeBinary = realDocker ? integrationBinary() : null;
    const directory = await mkdtemp(join(tmpdir(), "chapchoo-e2e-"));
    const ports = new Set<number>();
    let assembly: Assembly | null = null;
    try {
      while (ports.size < 4) {
        ports.add(await privatePort());
      }
      const [general, admin, generalWeb, adminWeb] = [...ports];
      if (!general || !admin || !generalWeb || !adminWeb) {
        throw new Error("Could not allocate isolated fixture ports");
      }
      assembly = new Assembly(
        directory,
        general,
        admin,
        generalWeb,
        adminWeb,
        packageRoot,
        docker,
        nativeBinary,
      );
      await assembly.start();
      return assembly;
    } catch (error) {
      if (assembly) {
        try {
          await assembly.close();
        } catch (cleanup) {
          throw new AggregateError(
            [error, cleanup],
            "Fixture startup and owned cleanup failed",
          );
        }
      } else {
        await rm(directory, { recursive: true, force: true });
      }
      throw new Error(
        `Assembled fixture startup failed: ${error instanceof Error ? safeMessage(error.message) : "Unknown startup failure"}`,
        { cause: error },
      );
    }
  }
  private webEnvironment(url: string, port: number): NodeJS.ProcessEnv {
    return {
      ...safeEnvironment(),
      NODE_ENV: "production",
      SWC_NATIVE_BINDING_CACHE: join(this.directory, "swc-cache"),
      CHAPCHOO_WEB_HOST: "127.0.0.1",
      CHAPCHOO_PUBLIC_URL: url,
      CHAPCHOO_GENERAL_WEB_URL: this.generalUrl,
      CHAPCHOO_ADMIN_WEB_URL: this.adminUrl,
      PORT: new URL(url).port,
      CHAPCHOO_BACKEND_URL: `http://127.0.0.1:${port}`,
    };
  }
  private async start(): Promise<void> {
    if (
      !/^[A-Za-z0-9/_-]+$/.test(this.directory) ||
      this.directory.length > 70
    ) {
      throw new Error("Private PostgreSQL socket path is unsuitable");
    }
    await mkdir(join(this.directory, "swc-cache"), { mode: 0o700 });
    const data = join(this.directory, "data");
    const socket = join(this.directory, "socket");
    await mkdir(socket, { mode: 0o700 });
    // Required tools fail honestly. There is no SQLite fallback or skip path.
    await this.owner.run(
      "Private PostgreSQL initialization",
      "initdb",
      [
        "-D",
        data,
        "-U",
        "fixture",
        "--auth=trust",
        "--no-locale",
        "--encoding=UTF8",
      ],
      repository,
      safeEnvironment(),
      30_000,
    );
    this.postgresAttempted = true;
    await this.owner.run(
      "Private PostgreSQL startup",
      "pg_ctl",
      [
        "-D",
        data,
        "-l",
        join(this.directory, "postgres.log"),
        "-o",
        `-k ${socket} -h '' -p 5432`,
        "-w",
        "-t",
        "30",
        "start",
      ],
      repository,
      safeEnvironment(),
      35_000,
    );
    const backendEnv = {
      ...safeEnvironment(),
      // The fixture runner needs development tools; the installed command does
      // not. Its deployment PATH deliberately contains no host Node/pnpm/Turbo.
      ...(this.packageRoot
        ? { PATH: join(this.directory, "no-host-tools") }
        : {}),
      CHAPCHOO_POSTGRES_URL: `postgresql://fixture@localhost/postgres?host=${encodeURIComponent(socket)}`,
      CHAPCHOO_SECRET_KEY: `${secret()}=`,
    };
    const rust = join(repository, "rust");
    if (!this.packageRoot && !this.nativeBinary) {
      await this.owner.run(
        "Product binary build",
        "cargo",
        ["build", "--locked", "-p", "chapchoo"],
        rust,
        buildEnvironment(),
        300_000,
      );
    }
    const binary =
      this.nativeBinary ??
      (this.packageRoot
        ? join(this.packageRoot, "chapchoo")
        : join(rust, "target/debug/chapchoo"));
    await this.owner.run(
      "Product schema preparation",
      binary,
      ["prepare"],
      repository,
      backendEnv,
      60_000,
    );
    const bootstrapProviders = this.docker
      ? ["fixture-provider", this.docker.provider]
      : ["fixture-provider"];
    for (const id of bootstrapProviders) {
      await this.owner.run(
        "Provider bootstrap credential issuance",
        binary,
        [
          "provider-credential",
          "--provider-id",
          id,
          "--output-file",
          this.providerCredentialFile(id),
        ],
        repository,
        {
          ...safeEnvironment(),
          CHAPCHOO_SECRET_KEY: backendEnv.CHAPCHOO_SECRET_KEY,
        },
        15_000,
      );
    }
    const adminBackend = this.owner.start(
      "Admin-only API server",
      binary,
      ["serve", "--groups", "admin", "--listen", `127.0.0.1:${this.adminPort}`],
      repository,
      {
        ...backendEnv,
        CHAPCHOO_SETUP_PROOF: this.setupProof,
        CHAPCHOO_GENERAL_WEB_URL: this.generalUrl,
        RUST_LOG: "chapchoo=info",
      },
    );
    await adminBackend.ready(
      /Selected backend service groups are ready/,
      60_000,
    );
    this.product = this.owner.start(
      "Product foreground server",
      binary,
      [
        "serve",
        "--groups",
        "client,control",
        "--listen",
        `${this.docker ? "0.0.0.0" : "127.0.0.1"}:${this.generalPort}`,
        ...(this.packageRoot && !this.docker
          ? [
              "--web",
              "both",
              "--general-web-listen",
              `127.0.0.1:${new URL(this.generalUrl).port}`,
              "--admin-web-listen",
              `127.0.0.1:${new URL(this.adminUrl).port}`,
            ]
          : []),
      ],
      repository,
      {
        ...backendEnv,
        CHAPCHOO_SETUP_PROOF: this.setupProof,
        CHAPCHOO_GENERAL_WEB_URL: this.generalUrl,
        CHAPCHOO_ADMIN_WEB_URL: this.adminUrl,
        CHAPCHOO_GENERAL_BACKEND_URL: `http://127.0.0.1:${this.generalPort}`,
        CHAPCHOO_ADMIN_BACKEND_URL: `http://127.0.0.1:${this.adminPort}`,
        CHAPCHOO_PROVIDER_BOOTSTRAP_CONFIG: JSON.stringify({
          secretKeyProviderIds: bootstrapProviders,
        }),
        RUST_LOG: "chapchoo=info",
      },
    );
    await this.product.ready(
      this.packageRoot && !this.docker
        ? /Selected components are ready/
        : /Selected backend service groups are ready/,
      60_000,
    );
    this.productReady = true;
    if (this.packageRoot || this.docker) {
      return;
    }
    for (const [name, url, backendPort] of [
      ["web", this.generalUrl, this.generalPort],
      ["admin-web", this.adminUrl, this.adminPort],
    ] satisfies [string, string, number][]) {
      const cwd = join(typescript, "apps", name);
      const env = this.webEnvironment(url, backendPort);
      await this.owner.run(
        `${name} font assets`,
        process.execPath,
        [join(typescript, "scripts/stage-web-fonts.mjs"), name],
        cwd,
        safeEnvironment(),
        30_000,
      );
      await this.owner.run(
        `${name} production build`,
        join(cwd, "node_modules/.bin/next"),
        ["build", "--webpack"],
        cwd,
        {
          ...safeEnvironment(),
          SWC_NATIVE_BINDING_CACHE: join(this.directory, "swc-cache"),
        },
        120_000,
      );
      const web = this.owner.start(
        `${name} production listener`,
        process.execPath,
        ["server.mjs"],
        cwd,
        env,
      );
      await web.ready(/Ready in/, 60_000);
    }
  }
  private providerCredentialFile(id: string): string {
    if (!/^[a-zA-Z0-9_-]+$/.test(id)) {
      throw new Error("Invalid fixture Provider identity");
    }
    return join(this.directory, `provider-${id}.credential`);
  }
  async initializeAdministrator(): Promise<void> {
    await seedAdministrator(
      `127.0.0.1:${this.adminPort}`,
      this.setupProof,
      this.administrator,
    );
  }
  async startComputerProvider(): Promise<void> {
    if (this.kubernetes) throw new Error("Computer fixture is already active");
    const binary = this.packageRoot
      ? join(this.packageRoot, "chapchoo")
      : join(repository, "rust/target/debug/chapchoo");
    const env = {
      ...safeEnvironment(),
      ...(this.packageRoot
        ? { PATH: join(this.directory, "no-host-tools") }
        : {}),
    };
    this.kubernetes = await SyntheticKubernetes.create(
      (session, key, endpoint) =>
        this.owner.start(
          "Execution Computer Connector",
          binary,
          ["computer-connector"],
          repository,
          {
            ...env,
            CHAPCHOO_CONTROL_ENDPOINT: endpoint,
            CHAPCHOO_CONNECTOR_SESSION_ID: session,
            CHAPCHOO_CONNECTOR_INITIAL_KEY: key,
          },
        ),
    );
    const kubeconfig = join(this.directory, "fixture-kubeconfig");
    await writeFile(
      kubeconfig,
      JSON.stringify({
        apiVersion: "v1",
        kind: "Config",
        "current-context": "fixture",
        clusters: [
          { name: "fixture", cluster: { server: this.kubernetes.endpoint } },
        ],
        contexts: [
          {
            name: "fixture",
            context: {
              cluster: "fixture",
              user: "fixture",
              namespace: "fixture",
            },
          },
        ],
        users: [{ name: "fixture", user: {} }],
      }),
      { mode: 0o600 },
    );
    this.owner.start(
      "Computer Provider",
      binary,
      ["computer-provider", "--kubeconfig", kubeconfig],
      repository,
      {
        ...env,
        CHAPCHOO_CONTROL_ENDPOINT: `http://127.0.0.1:${this.generalPort}`,
        CHAPCHOO_PROVIDER_ID: "fixture-provider",
        CHAPCHOO_PROVIDER_NAME: "Acceptance Kubernetes",
        CHAPCHOO_PROVIDER_AUTH_METHOD: "secret-key",
        CHAPCHOO_PROVIDER_AUTH_FILE:
          this.providerCredentialFile("fixture-provider"),
        CHAPCHOO_KUBERNETES_NAMESPACE: "fixture",
        CHAPCHOO_COMPUTER_IMAGE: "fixture/connector:acceptance",
        CHAPCHOO_COMPUTER_SERVICE_ACCOUNT: "fixture",
      },
    );
  }
  get generalRpcEndpoint(): string {
    return `127.0.0.1:${this.generalPort}`;
  }
  async startDockerProvider(): Promise<OwnedChild> {
    if (!this.docker) {
      throw new Error("Real Docker Assembly was not explicitly selected");
    }
    await this.docker.prepare(this.generalPort);
    const binary =
      this.nativeBinary ??
      (this.packageRoot
        ? join(this.packageRoot, "chapchoo")
        : join(repository, "rust/target/debug/chapchoo"));
    return this.owner.start(
      "Native Docker Computer Provider",
      binary,
      ["docker-provider"],
      repository,
      {
        ...safeEnvironment(),
        ...(this.packageRoot
          ? { PATH: join(this.directory, "no-host-tools") }
          : {}),
        CHAPCHOO_CONTROL_ENDPOINT: `http://127.0.0.1:${this.generalPort}`,
        CHAPCHOO_CONTAINER_CONTROL_ENDPOINT: this.docker.controlEndpoint,
        CHAPCHOO_PROVIDER_ID: this.docker.provider,
        CHAPCHOO_PROVIDER_NAME: "Acceptance Docker",
        CHAPCHOO_PROVIDER_AUTH_METHOD: "secret-key",
        CHAPCHOO_PROVIDER_AUTH_FILE: this.providerCredentialFile(
          this.docker.provider,
        ),
        CHAPCHOO_DOCKER_HOST: pathToFileURL(
          this.docker.config.socket,
        ).href.replace(/^file:/, "unix:"),
        CHAPCHOO_COMPUTER_IMAGE: this.docker.config.image,
        CHAPCHOO_COMPUTER_IMAGE_PULL_POLICY: "Never",
      },
    );
  }
  get savedAllocations(): number {
    return this.kubernetes?.savedAllocations ?? 0;
  }
  async seedSecondUser(): Promise<void> {
    await seedRegisteredUser(
      `127.0.0.1:${this.generalPort}`,
      `127.0.0.1:${this.adminPort}`,
      this.administrator,
      this.second,
    );
  }
  async close(): Promise<void> {
    if (this.closed) {
      return;
    }
    this.closed = true;
    const failures: Error[] = [];
    const record = (stage: string, error: unknown): void => {
      failures.push(
        new Error(
          `${stage}: ${error instanceof Error ? safeMessage(error.message) : "Unknown failure"}`,
        ),
      );
    };
    try {
      await this.owner.stopRunning();
      if (this.productReady) {
        await this.product?.finished(5_000);
      }
    } catch (error) {
      record("Owned process shutdown", error);
    }
    try {
      await this.kubernetes?.close();
    } catch (error) {
      record("Kubernetes fixture cleanup", error);
    }
    try {
      await this.docker?.close();
    } catch (error) {
      record("Docker fixture cleanup", error);
    }
    // PostgreSQL is managed only by this fixture's exact private data directory.
    let postgresPresent = false;
    if (this.postgresAttempted) {
      try {
        await access(join(this.directory, "data", "postmaster.pid"));
        postgresPresent = true;
      } catch (error) {
        if (!(
          error instanceof Error &&
          "code" in error &&
          error.code === "ENOENT"
        )) {
          record("PostgreSQL presence verification", error);
        }
      }
    }
    if (postgresPresent) {
      try {
        await this.owner.run(
          "Private PostgreSQL shutdown",
          "pg_ctl",
          [
            "-D",
            join(this.directory, "data"),
            "-m",
            "fast",
            "-w",
            "-t",
            "30",
            "stop",
          ],
          repository,
          safeEnvironment(),
          35_000,
        );
      } catch {
        try {
          await this.owner.run(
            "Private PostgreSQL immediate shutdown",
            "pg_ctl",
            [
              "-D",
              join(this.directory, "data"),
              "-m",
              "immediate",
              "-w",
              "-t",
              "5",
              "stop",
            ],
            repository,
            safeEnvironment(),
            10_000,
          );
        } catch (error) {
          record("PostgreSQL shutdown", error);
        }
      }
    }
    try {
      await this.owner.stopAll();
    } catch (error) {
      record("Owned process reaping", error);
    }
    if (failures.length === 0) {
      await rm(this.directory, { recursive: true, force: true });
    }
    if (failures.length > 0) {
      throw new AggregateError(
        failures,
        "Owned fixture cleanup failed; private directory retained",
      );
    }
  }
}
