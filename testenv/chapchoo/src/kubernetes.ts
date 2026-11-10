import { createServer } from "node:http";
import { randomUUID } from "node:crypto";
import { EventEmitter, once } from "node:events";
import type { IncomingMessage, ServerResponse } from "node:http";
import type { OwnedChild } from "./processes.js";

interface ManagedObject {
  apiVersion: string;
  kind: string;
  metadata: {
    name: string;
    namespace: string;
    uid?: string;
    resourceVersion?: string;
    annotations: Record<string, string>;
  };
  spec?: unknown;
  immutable?: boolean;
  type?: string;
  data?: Record<string, string>;
  status?: unknown;
}
function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
function strings(value: unknown): value is Record<string, string> {
  return (
    record(value) &&
    Object.values(value).every((item) => typeof item === "string")
  );
}
function object(value: unknown): ManagedObject {
  if (
    !record(value) ||
    typeof value.apiVersion !== "string" ||
    typeof value.kind !== "string" ||
    !record(value.metadata) ||
    typeof value.metadata.name !== "string" ||
    typeof value.metadata.namespace !== "string" ||
    !strings(value.metadata.annotations) ||
    (value.data !== void 0 && !strings(value.data))
  )
    throw new Error("Malformed synthetic Kubernetes object");
  return {
    apiVersion: value.apiVersion,
    kind: value.kind,
    metadata: {
      name: value.metadata.name,
      namespace: value.metadata.namespace,
      annotations: value.metadata.annotations,
    },
    spec: value.spec,
    ...(typeof value.immutable === "boolean"
      ? { immutable: value.immutable }
      : {}),
    ...(typeof value.type === "string" ? { type: value.type } : {}),
    ...(value.data ? { data: value.data } : {}),
  };
}
async function body(request: IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of request) {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk));
    size += bytes.length;
    if (size > 65536)
      throw new Error("Synthetic Kubernetes body exceeds bound");
    chunks.push(bytes);
  }
  const decoded: unknown = JSON.parse(Buffer.concat(chunks).toString());
  return decoded;
}

/** Isolated SDK transport fixture; not a real cluster, storage driver or CNI. */
export class SyntheticKubernetes {
  private readonly objects = new Map<string, ManagedObject>();
  private readonly watches = new Set<ServerResponse>();
  private readonly connectors = new Map<string, OwnedChild>();
  private readonly allocations = new Set<string>();
  private executions = 0;
  private shutdownBarrier: {
    entered: () => void;
    wait: Promise<void>;
    release: () => void;
  } | null = null;
  private server = createServer((request, response) => {
    void this.handle(request, response).catch(() => {
      if (!response.headersSent)
        response.writeHead(500, { "content-type": "application/json" });
      response.end(
        JSON.stringify({
          apiVersion: "v1",
          kind: "Status",
          code: 500,
          reason: "InternalError",
        }),
      );
    });
  });
  private constructor(
    private readonly startConnector: (
      session: string,
      key: string,
      endpoint: string,
      computerId: string,
    ) => Promise<OwnedChild>,
  ) {}
  static async create(
    startConnector: (
      session: string,
      key: string,
      endpoint: string,
      computerId: string,
    ) => Promise<OwnedChild>,
  ): Promise<SyntheticKubernetes> {
    const fixture = new SyntheticKubernetes(startConnector);
    await new Promise<void>((resolve, reject) => {
      fixture.server.once("error", reject);
      fixture.server.listen(0, "127.0.0.1", resolve);
    });
    return fixture;
  }
  get endpoint(): string {
    const address = this.server.address();
    if (!address || typeof address === "string")
      throw new Error("Synthetic Kubernetes listener unavailable");
    return `http://127.0.0.1:${address.port}`;
  }
  get savedAllocations(): number {
    return this.allocations.size;
  }
  get executionAllocations(): number {
    return this.executions;
  }
  pauseNextShutdown(): { started: Promise<void>; release: () => void } {
    if (this.shutdownBarrier)
      throw new Error("A shutdown barrier is already active");
    const events = new EventEmitter();
    const started = once(events, "started").then(() => {});
    const released = once(events, "released").then(() => {});
    this.shutdownBarrier = {
      entered: () => {
        events.emit("started");
      },
      wait: released,
      release: () => {
        events.emit("released");
      },
    };
    return {
      started,
      release: () => {
        events.emit("released");
      },
    };
  }
  private reply(response: ServerResponse, code: number, value: unknown): void {
    response.writeHead(code, { "content-type": "application/json" });
    response.end(JSON.stringify(value));
  }
  private async handle(
    request: IncomingMessage,
    response: ServerResponse,
  ): Promise<void> {
    const url = new URL(request.url ?? "/", this.endpoint);
    if (
      !/^\/api\/v1\/namespaces\/fixture\/(pods|secrets|persistentvolumeclaims)(\/[^/]+)?$/.test(
        url.pathname,
      )
    ) {
      this.reply(response, 404, {
        apiVersion: "v1",
        kind: "Status",
        code: 404,
        reason: "NotFound",
      });
      return;
    }
    if (request.method === "GET" && url.searchParams.get("watch") === "true") {
      response.writeHead(200, { "content-type": "application/json" });
      response.flushHeaders();
      this.watches.add(response);
      response.once("close", () => this.watches.delete(response));
      return;
    }
    if (request.method === "POST") {
      const created = object(await body(request));
      const path = `${url.pathname}/${created.metadata.name}`;
      if (this.objects.has(path)) {
        this.reply(response, 409, {
          apiVersion: "v1",
          kind: "Status",
          code: 409,
          reason: "AlreadyExists",
        });
        return;
      }
      created.metadata.uid = randomUUID();
      created.metadata.resourceVersion = randomUUID();
      if (created.kind === "PersistentVolumeClaim")
        this.allocations.add(created.metadata.uid);
      if (created.kind === "Pod") {
        this.executions += 1;
        const bootstrap = [...this.objects.values()].find(
          (item) =>
            item.kind === "Secret" &&
            item.metadata.annotations["chapchoo.io/execution"] ===
              created.metadata.annotations["chapchoo.io/execution"],
        );
        if (!bootstrap?.data)
          throw new Error("Synthetic Pod bootstrap missing");
        const decode = (key: string): string => {
          const encoded = bootstrap.data?.[key];
          if (!encoded) throw new Error("Synthetic bootstrap input missing");
          return Buffer.from(encoded, "base64").toString();
        };
        this.connectors.set(
          path,
          await this.startConnector(
            decode("session-id"),
            decode("initial-key"),
            decode("control-endpoint"),
            created.metadata.annotations["chapchoo.io/computer"] ?? "",
          ),
        );
        created.status = { phase: "Running" };
      }
      this.objects.set(path, created);
      this.reply(response, 201, created);
      return;
    }
    const current = this.objects.get(url.pathname);
    if (!current) {
      this.reply(response, 404, {
        apiVersion: "v1",
        kind: "Status",
        code: 404,
        reason: "NotFound",
      });
      return;
    }
    if (request.method === "DELETE") {
      const deletion = await body(request);
      if (
        !record(deletion) ||
        !record(deletion.preconditions) ||
        deletion.preconditions.uid !== current.metadata.uid
      ) {
        this.reply(response, 409, {
          apiVersion: "v1",
          kind: "Status",
          code: 409,
          reason: "Conflict",
        });
        return;
      }
      if (current.kind === "Pod") {
        const barrier = this.shutdownBarrier;
        if (barrier) {
          barrier.entered();
          await barrier.wait;
          this.shutdownBarrier = null;
        }
        await this.connectors.get(url.pathname)?.stop();
        this.connectors.delete(url.pathname);
        const terminal = {
          ...current,
          status: {
            phase: "Succeeded",
            containerStatuses: [
              {
                name: "computer-connector",
                image: "fixture",
                imageID: "fixture-image",
                ready: false,
                restartCount: 0,
                state: {
                  terminated: {
                    containerID: `fixture://${current.metadata.uid}`,
                    exitCode: 0,
                    reason: "Completed",
                    finishedAt: new Date().toISOString(),
                  },
                },
              },
            ],
          },
        };
        for (const watch of this.watches)
          watch.write(
            `${JSON.stringify({ type: "DELETED", object: terminal })}\n`,
          );
      }
      this.objects.delete(url.pathname);
      this.reply(response, 200, {
        apiVersion: "v1",
        kind: "Status",
        status: "Success",
      });
      return;
    }
    this.reply(response, 200, current);
  }
  async close(): Promise<void> {
    this.shutdownBarrier?.release();
    for (const watch of this.watches) watch.end();
    for (const connector of this.connectors.values()) await connector.stop();
    this.connectors.clear();
    await new Promise<void>((resolve, reject) =>
      this.server.close((error) => (error ? reject(error) : resolve())),
    );
  }
}
