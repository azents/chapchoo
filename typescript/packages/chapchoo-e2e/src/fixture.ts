import { access, mkdtemp, mkdir, rm } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test as base } from "@playwright/test";
import {
  buildEnvironment,
  installedPackageRoot,
  safeEnvironment,
} from "./config/index.js";
import { ProcessOwner } from "./processes.js";
import { fixtureUser, secret, safeMessage } from "./safety.js";
import { seedRegisteredUser } from "./seed.js";
import type { FixtureUser } from "./safety.js";
import type { OwnedChild } from "./processes.js";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
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
  ) {
    this.owner = new ProcessOwner(join(directory, "owned-processes.jsonl"));
    this.generalUrl = `http://127.0.0.1:${generalWebPort}${packageRoot ? "" : "/app"}`;
    this.adminUrl = `http://127.0.0.1:${adminWebPort}${packageRoot ? "" : "/admin"}`;
  }
  static async create(): Promise<Assembly> {
    const packageRoot = installedPackageRoot();
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
      CHAPCHOO_WEB_HOST: "127.0.0.1",
      CHAPCHOO_PUBLIC_URL: url,
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
    if (!this.packageRoot) {
      await this.owner.run(
        "Product binary build",
        "cargo",
        ["build", "--locked", "-p", "chapchoo"],
        rust,
        buildEnvironment(),
        300_000,
      );
    }
    const binary = this.packageRoot
      ? join(this.packageRoot, "chapchoo")
      : join(rust, "target/debug/chapchoo");
    await this.owner.run(
      "Product schema preparation",
      binary,
      ["prepare"],
      repository,
      backendEnv,
      60_000,
    );
    this.product = this.owner.start(
      "Product foreground server",
      binary,
      [
        "serve",
        "--role",
        "both",
        "--general-listen",
        `127.0.0.1:${this.generalPort}`,
        "--admin-listen",
        `127.0.0.1:${this.adminPort}`,
        ...(this.packageRoot
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
        RUST_LOG: "chapchoo=info",
      },
    );
    await this.product.ready(
      this.packageRoot
        ? /Selected components are ready/
        : /Selected backend roles are ready/,
      60_000,
    );
    this.productReady = true;
    if (this.packageRoot) {
      return;
    }
    for (const [name, url, backendPort] of [
      ["chapchoo-web", this.generalUrl, this.generalPort],
      ["chapchoo-admin-web", this.adminUrl, this.adminPort],
    ] satisfies [string, string, number][]) {
      const cwd = join(typescript, "apps", name);
      const env = this.webEnvironment(url, backendPort);
      await this.owner.run(
        `${name} production build`,
        join(cwd, "node_modules/.bin/next"),
        ["build"],
        cwd,
        env,
        120_000,
      );
      const web = this.owner.start(
        `${name} production listener`,
        join(cwd, "node_modules/.bin/next"),
        ["start", "--hostname", "127.0.0.1", "--port", new URL(url).port],
        cwd,
        env,
      );
      await web.ready(/Ready in/, 60_000);
    }
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
    let failed = false;
    try {
      await this.owner.stopRunning();
      if (this.productReady) {
        await this.product?.finished(5_000);
      }
    } catch {
      failed = true;
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
          failed = true;
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
        } catch {
          failed = true;
        }
      }
    }
    try {
      await this.owner.stopAll();
    } catch {
      failed = true;
    }
    if (!failed) {
      await rm(this.directory, { recursive: true, force: true });
    }
    if (failed) {
      throw new Error(
        "Owned fixture cleanup failed; private directory retained",
      );
    }
  }
}

export const test = base.extend<object, { assembly: Assembly }>({
  assembly: [
    async ({ playwright }, use) => {
      void playwright;
      const assembly = await Assembly.create();
      try {
        await use(assembly);
      } finally {
        await assembly.close();
      }
    },
    { scope: "worker", timeout: 660_000 },
  ],
});
export { expect } from "@playwright/test";
