import Docker from "dockerode";
import { secret } from "./safety.js";
import {
  cleanupCanary,
  cleanupCanaryVolume,
  recoverCanaryId,
} from "./canaryCleanup.js";
import type { CanaryState, CanaryVolumeState } from "./canaryCleanup.js";
import type { DockerFixtureConfig } from "./config/index.js";
import { runningSuccessor } from "./dockerObservation.js";
import type { NativeComputer, NativeObservation } from "./dockerObservation.js";
export interface DockerEvidence {
  daemon: string;
  engine: string;
  api: string;
  architecture: string;
  image: string;
}
function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
function missing(error: unknown): boolean {
  return object(error) && error.statusCode === 404;
}
async function native<T>(operation: () => Promise<T>): Promise<T> {
  try {
    return await operation();
  } catch {
    throw new Error(
      "Owned Docker fixture operation failed; native response withheld",
    );
  }
}
function labelsMatch(
  actual: Record<string, string> | undefined,
  expected: Record<string, string>,
): boolean {
  return (
    actual !== undefined &&
    Object.entries(expected).every(([key, value]) => actual[key] === value)
  );
}

/** Owns native prerequisites and test-only file IO, not product lifecycle logic. */
export class DockerFixture {
  readonly registrationKey = `docker-${secret().slice(0, 16).toLowerCase()}`;
  private canonicalProvider: string | null = null;
  private readonly run = `chapchoo-e2e-${secret().slice(0, 16).toLowerCase()}`;
  private readonly sdk: Docker;
  private daemon = "";
  private image = "";
  private canary: Docker.Container | null = null;
  private canaryVolume: Docker.Volume | null = null;
  private canaryId = "";
  private volumeName = "";
  private canaryVolumeCreated = false;
  private canaryCreated = false;
  private initialized = false;
  private uncertain:
    "volume-create" | "container-create" | "canary-start" | null = null;
  evidence: DockerEvidence | null = null;
  controlEndpoint = "";
  constructor(readonly config: DockerFixtureConfig) {
    this.sdk = new Docker({
      socketPath: config.socket,
      host: "",
      port: 0,
      protocol: "http",
      timeout: 10_000,
    });
  }
  private get canaryLabels(): Record<string, string> {
    return { "chapchoo.test/run": this.run };
  }
  private get productLabels(): string[] {
    return [`chapchoo.io/provider=${this.providerId}`];
  }
  /** Bind the identity returned by registration before observing new allocations. */
  bindProvider(providerId: string): void {
    if (
      !providerId ||
      providerId.length > 128 ||
      this.canonicalProvider !== null
    ) {
      throw new Error(
        "Canonical fixture Provider identity must be bound exactly once",
      );
    }
    this.canonicalProvider = providerId;
  }
  private get providerId(): string {
    if (this.canonicalProvider === null) {
      throw new Error("Canonical fixture Provider identity is not bound");
    }
    return this.canonicalProvider;
  }
  private async verifyDaemon(): Promise<void> {
    const info: unknown = await native(() => this.sdk.info());
    if (
      !object(info) ||
      typeof info.ID !== "string" ||
      !info.ID ||
      (this.daemon && info.ID !== this.daemon)
    ) {
      throw new Error("Docker fixture daemon identity is missing or changed");
    }
    this.daemon = info.ID;
  }
  async prepare(controlPort: number): Promise<void> {
    await this.verifyDaemon();
    const version = await native(() => this.sdk.version());
    if (version.Os !== "linux" || !["amd64", "arm64"].includes(version.Arch)) {
      throw new Error(
        "Docker acceptance requires a supported Linux daemon architecture",
      );
    }
    const image = await native(() =>
      this.sdk.getImage(this.config.image).inspect(),
    );
    if (
      !/^sha256:[a-f0-9]{64}$/.test(image.Id) ||
      image.Os !== "linux" ||
      image.Architecture !== version.Arch
    ) {
      throw new Error(
        "Docker acceptance requires the current architecture-matched built Linux image",
      );
    }
    this.image = image.Id;
    const bridge = await native(() => this.sdk.getNetwork("bridge").inspect());
    const host = this.config.controlHost ?? bridge.IPAM?.Config?.[0]?.Gateway;
    if (!host || !/^[a-zA-Z0-9.-]+$/.test(host) || bridge.Internal) {
      throw new Error(
        "Docker acceptance needs an existing outbound bridge and explicit reachable Control route",
      );
    }
    this.controlEndpoint = `http://${host}:${controlPort}`;
    this.evidence = {
      daemon: this.daemon,
      engine: version.Version,
      api: version.ApiVersion,
      architecture: version.Arch,
      image: this.image,
    };
    this.initialized = true;
    // Canary objects use a disjoint fixture label, never Provider ownership.
    this.volumeName = `${this.run}-volume`;
    try {
      await this.sdk.getVolume(this.volumeName).inspect();
      throw new Error("Canary volume name already exists");
    } catch (error) {
      if (!missing(error)) {
        throw new Error("Canary volume absence was not verified");
      }
    }
    this.uncertain = "volume-create";
    this.canaryVolume = await native(() =>
      this.sdk.createVolume({
        Name: this.volumeName,
        Driver: "local",
        DriverOpts: {},
        Labels: this.canaryLabels,
      }),
    );
    this.canaryVolumeCreated = true;
    this.uncertain = "container-create";
    this.canary = await native(() =>
      this.sdk.createContainer({
        name: `${this.run}-container`,
        Image: this.image,
        User: "10001:10001",
        Entrypoint: ["/bin/sh"],
        Cmd: ["-c", "exec sleep 86400"],
        Labels: this.canaryLabels,
        HostConfig: {
          ReadonlyRootfs: true,
          CapDrop: ["ALL"],
          SecurityOpt: ["no-new-privileges=true"],
          NetworkMode: "none",
          Memory: 64 * 1024 * 1024,
          NanoCpus: 100_000_000,
          RestartPolicy: { Name: "no" },
        },
      }),
    );
    this.canaryId = this.canary.id;
    this.canaryCreated = true;
    this.uncertain = "canary-start";
    await native(() => this.canaryRequired().start());
    this.uncertain = null;
  }
  private canaryRequired(): Docker.Container {
    if (!this.canary) {
      throw new Error("Canary container is unavailable");
    }
    return this.canary;
  }
  async current(): Promise<NativeComputer> {
    const observed = await this.observe();
    if (observed.type !== "present" || observed.status !== "running") {
      throw new Error("A verified running native execution is not present");
    }
    return observed.native;
  }
  async successor(previous: NativeComputer): Promise<NativeComputer | null> {
    return runningSuccessor(previous, await this.observe());
  }
  private async observe(): Promise<NativeObservation> {
    await this.verifyDaemon();
    const containers = await native(() =>
      this.sdk.listContainers({
        all: true,
        filters: JSON.stringify({ label: this.productLabels }),
      }),
    );
    const volumes = await native(() =>
      this.sdk.listVolumes({
        filters: JSON.stringify({ label: this.productLabels }),
      }),
    );
    if (volumes.Volumes?.length !== 1 || !volumes.Volumes[0]) {
      throw new Error(
        "Expected one native execution and its one original volume",
      );
    }
    const saved = volumes.Volumes[0];
    if (
      saved.Driver !== "local" ||
      saved.Scope !== "local" ||
      !labelsMatch(saved.Labels, { "chapchoo.io/provider": this.providerId })
    ) {
      throw new Error("Original volume ownership is unverified");
    }
    if (containers.length === 0) {
      return { type: "absent", daemon: this.daemon, volume: saved.Name };
    }
    if (containers.length !== 1 || !containers[0]) {
      throw new Error("Unexpected native execution inventory");
    }
    const selected = containers[0];
    let inspected: Docker.ContainerInspectInfo;
    try {
      inspected = await this.sdk.getContainer(selected.Id).inspect();
    } catch (error) {
      // A read-only list/inspect race can observe the approved removal gap.
      // Other native failures still stop the poll immediately.
      if (missing(error)) {
        return { type: "absent", daemon: this.daemon, volume: saved.Name };
      }
      throw new Error(
        "Native replacement inspection failed; response withheld",
      );
    }
    const mount = inspected.HostConfig.Mounts?.[0];
    if (
      !/^[a-f0-9]{64}$/.test(inspected.Id) ||
      inspected.Id !== selected.Id ||
      inspected.Image !== this.image ||
      mount?.Source !== saved.Name ||
      mount.Target !== "/workspace" ||
      inspected.Mounts.length !== 1 ||
      !labelsMatch(inspected.Config.Labels, {
        "chapchoo.io/provider": this.providerId,
      }) ||
      saved.Driver !== "local" ||
      saved.Scope !== "local"
    ) {
      throw new Error(
        "Native Computer evidence does not match the fixture's reserved execution",
      );
    }
    const state = inspected.State;
    const status = state.Status;
    if (
      (status !== "created" &&
        status !== "running" &&
        status !== "exited" &&
        status !== "removing") ||
      state.Running !== (status === "running") ||
      state.Dead ||
      state.Paused ||
      state.Restarting ||
      state.Error
    ) {
      throw new Error("Native execution is not in an admitted readiness state");
    }
    const value: NativeComputer = {
      container: inspected.Id,
      volume: saved.Name,
      daemon: this.daemon,
      user: inspected.Config.User,
      noCopy: mount.VolumeOptions?.NoCopy ?? false,
    };
    return { type: "present", native: value, status };
  }
  private async exec(target: NativeComputer, script: string): Promise<string> {
    await this.verifyDaemon();
    const inspected = await native(() =>
      this.sdk.getContainer(target.container).inspect(),
    );
    if (
      !labelsMatch(inspected.Config.Labels, {
        "chapchoo.io/provider": this.providerId,
      }) ||
      inspected.Id !== target.container ||
      inspected.Mounts[0]?.Name !== target.volume
    ) {
      throw new Error(
        "File fixture cannot enter a foreign or replaced native execution",
      );
    }
    const execution = await native(() =>
      this.sdk.getContainer(target.container).exec({
        AttachStdout: true,
        AttachStderr: true,
        AttachStdin: false,
        Tty: true,
        User: "10001:10001",
        Cmd: ["/bin/sh", "-c", script],
      }),
    );
    const stream = await native(() =>
      execution.start({ Detach: false, Tty: true }),
    );
    let output = "";
    const timer = setTimeout(
      () => stream.destroy(new Error("Bounded marker exec deadline exceeded")),
      10_000,
    );
    try {
      for await (const chunk of stream) {
        if (!(chunk instanceof Buffer)) {
          throw new Error("Unexpected marker output");
        }
        output += chunk.toString();
        if (output.length > 4096) {
          stream.destroy();
          throw new Error("Unbounded marker output");
        }
      }
    } finally {
      clearTimeout(timer);
      stream.destroy();
    }
    const result = await native(() => execution.inspect());
    if (
      result.Running ||
      result.ExitCode !== 0 ||
      result.ContainerID !== target.container
    ) {
      throw new Error("Non-root saved marker operation failed");
    }
    return output.trim();
  }
  async writeMarker(target: NativeComputer): Promise<void> {
    await this.exec(
      target,
      "test \"$(id -u)\" = 10001 && printf 'saved-original-data' > /workspace/acceptance-marker",
    );
  }
  async readMarker(target: NativeComputer): Promise<string> {
    return this.exec(target, "cat /workspace/acceptance-marker");
  }
  /** Empty original storage must stay empty on a successor (no image refill). */
  async clearSaved(target: NativeComputer): Promise<void> {
    await this.exec(target, "rm /workspace/acceptance-marker");
  }
  async savedEmpty(target: NativeComputer): Promise<boolean> {
    return (
      (await this.exec(target, "find /workspace -mindepth 1 -print -quit")) ===
      ""
    );
  }
  async containerAbsent(id: string): Promise<boolean> {
    if (!/^[a-f0-9]{64}$/.test(id)) {
      throw new Error("Invalid fixture native identity");
    }
    await this.verifyDaemon();
    try {
      await this.sdk.getContainer(id).inspect();
      return false;
    } catch (error) {
      if (missing(error)) {
        return true;
      }
      throw new Error("Native container absence is unverified");
    }
  }
  async canariesPresent(): Promise<boolean> {
    await this.verifyDaemon();
    if (!this.canary || !this.canaryVolume) {
      return false;
    }
    const container = await native(() => this.canaryRequired().inspect());
    const volume = await native(() => this.canaryVolumeRequired().inspect());
    return (
      container.Id === this.canaryId &&
      container.State.Running &&
      labelsMatch(container.Config.Labels, this.canaryLabels) &&
      volume.Name === this.volumeName &&
      labelsMatch(volume.Labels, this.canaryLabels)
    );
  }
  private canaryVolumeRequired(): Docker.Volume {
    if (!this.canaryVolume) {
      throw new Error("Canary volume is unavailable");
    }
    return this.canaryVolume;
  }
  async productAbsent(): Promise<boolean> {
    await this.verifyDaemon();
    const containers = await native(() =>
      this.sdk.listContainers({
        all: true,
        filters: JSON.stringify({ label: this.productLabels }),
      }),
    );
    const volumes = await native(() =>
      this.sdk.listVolumes({
        filters: JSON.stringify({ label: this.productLabels }),
      }),
    );
    return containers.length === 0 && (volumes.Volumes?.length ?? 0) === 0;
  }
  async close(): Promise<void> {
    if (!this.initialized) {
      return;
    }
    await this.verifyDaemon();
    let failed = this.uncertain !== null && this.uncertain !== "canary-start";
    let productAbsent = false;
    try {
      productAbsent = await this.productAbsent();
    } catch {
      failed = true;
    }
    if (!this.canary && this.uncertain === "container-create") {
      try {
        const name = `${this.run}-container`;
        const observed = await this.sdk.getContainer(name).inspect();
        this.canaryId = recoverCanaryId(
          {
            id: observed.Id,
            name: observed.Name,
            labels: observed.Config.Labels ?? {},
          },
          name,
          this.canaryLabels,
        );
        this.canary = this.sdk.getContainer(this.canaryId);
      } catch (error) {
        if (!missing(error)) {
          failed = true;
        }
      }
    }
    if (
      this.canary &&
      (this.canaryCreated || this.uncertain === "container-create")
    ) {
      try {
        const canary = this.canary;
        await cleanupCanary(
          {
            inspect: async (): Promise<CanaryState | null> => {
              try {
                const container = await canary.inspect();
                return {
                  id: container.Id,
                  running: container.State.Running,
                  labels: container.Config.Labels ?? {},
                };
              } catch (error) {
                if (missing(error)) {
                  return null;
                }
                throw new Error("Canary state is unverified");
              }
            },
            stop: async (): Promise<void> => {
              await canary.stop({ t: 5 });
            },
            remove: async (): Promise<void> => {
              await canary.remove({ force: false, v: false });
            },
          },
          this.canaryId,
          this.canaryLabels,
        );
      } catch {
        failed = true;
      }
    }
    if (this.canaryVolumeCreated || this.uncertain === "volume-create") {
      try {
        const volume = this.canaryVolume ?? this.sdk.getVolume(this.volumeName);
        await cleanupCanaryVolume(
          {
            inspect: async (): Promise<CanaryVolumeState | null> => {
              try {
                const observed = await volume.inspect();
                return {
                  name: observed.Name,
                  driver: observed.Driver,
                  scope: observed.Scope,
                  labels: observed.Labels ?? {},
                };
              } catch (error) {
                if (missing(error)) {
                  return null;
                }
                throw new Error("Canary volume state is unverified");
              }
            },
            remove: async (): Promise<void> => {
              await volume.remove({ force: false });
            },
          },
          this.volumeName,
          this.canaryLabels,
        );
      } catch {
        failed = true;
      }
    }
    if (!productAbsent || failed) {
      // Native product cleanup belongs to Control/Provider. Fixture failure
      // never force-deletes unknown product work to manufacture success.
      throw new Error(
        "Docker fixture cleanup is unconfirmed; owned resources require diagnosis",
      );
    }
  }
}
