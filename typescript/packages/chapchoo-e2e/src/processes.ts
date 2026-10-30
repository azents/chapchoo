import { spawn } from "node:child_process";
import { appendFileSync } from "node:fs";
import { diagnosticTail } from "./safety.js";
import type { ChildProcessWithoutNullStreams } from "node:child_process";

interface Exit {
  success: boolean;
}

export class OwnedChild {
  readonly child: ChildProcessWithoutNullStreams;
  readonly completion: Promise<Exit>;
  private output = "";
  private ended = false;
  private cleaned = false;
  private truncated = false;
  private listeners = new Set<() => void>();

  constructor(
    readonly label: string,
    command: string,
    args: string[],
    cwd: string,
    env: NodeJS.ProcessEnv,
    readonly role: "command" | "service",
  ) {
    this.child = spawn(command, args, {
      cwd,
      env,
      detached: true,
      stdio: "pipe",
    });
    this.child.stdin.end();
    this.completion = new Promise((resolve) => {
      this.child.once("error", () => {
        this.ended = true;
        resolve({ success: false });
        this.notify();
      });
      this.child.once("close", (code) => {
        this.ended = true;
        resolve({ success: code === 0 });
        this.notify();
      });
    });
    // Drain both pipes without publishing commands, errors or application output.
    for (const stream of [this.child.stdout, this.child.stderr]) {
      stream.on("data", (chunk: Buffer) => {
        const combined = this.output + chunk.toString();
        this.truncated ||= combined.length > 16_384;
        this.output = combined.slice(-16_384);
        this.notify();
      });
    }
  }
  private notify(): void {
    for (const listener of this.listeners) {
      listener();
    }
  }
  private failure(message: string): Error {
    const diagnostic = diagnosticTail(this.output, this.truncated);
    return new Error(
      `${this.label} ${message}${diagnostic ? `: ${diagnostic}` : "; no complete safe diagnostic"}`,
    );
  }
  async finished(timeout: number): Promise<void> {
    const result = await bounded(
      this.completion,
      timeout,
      `${this.label} deadline exceeded`,
    ).catch(() => {
      throw this.failure("deadline exceeded");
    });
    if (!result.success) {
      throw this.failure("failed");
    }
  }
  async ready(pattern: RegExp, timeout: number): Promise<void> {
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        cleanup();
        reject(this.failure("readiness deadline exceeded"));
      }, timeout);
      const cleanup = (): void => {
        clearTimeout(timer);
        this.listeners.delete(check);
      };
      const check = (): void => {
        if (this.ended) {
          cleanup();
          reject(this.failure("exited before readiness"));
        } else if (pattern.test(this.output)) {
          cleanup();
          resolve();
        }
      };
      this.listeners.add(check);
      check();
    });
  }
  get running(): boolean {
    return !this.cleaned && (this.role === "service" || !this.ended);
  }
  async stop(): Promise<void> {
    if (this.cleaned) {
      return;
    }
    if (this.ended && this.role === "command") {
      this.cleaned = true;
      return;
    }
    const pid = this.child.pid;
    if (!pid) {
      await this.completion;
      this.cleaned = true;
      return;
    }
    signalGroup(pid, "SIGTERM");
    if (!(await groupGone(pid, 30_000))) {
      signalGroup(pid, "SIGKILL");
      if (!(await groupGone(pid, 5_000))) {
        throw new Error(`${this.label} owned process group did not exit`);
      }
    }
    await bounded(this.completion, 5_000, `${this.label} was not reaped`);
    this.cleaned = true;
  }
}

function signalGroup(pid: number, signal: NodeJS.Signals): void {
  try {
    process.kill(-pid, signal);
  } catch (error) {
    if (!(
      error instanceof Error &&
      "code" in error &&
      error.code === "ESRCH"
    )) {
      throw new Error("Could not signal an owned process group");
    }
  }
}

function exists(pid: number): boolean {
  try {
    process.kill(-pid, 0);
    return true;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ESRCH") {
      return false;
    }
    throw new Error("Could not inspect an owned process group");
  }
}

/** Poll only authoritative process-group existence during finite shutdown. */
async function groupGone(pid: number, timeout: number): Promise<boolean> {
  const deadline = Date.now() + timeout;
  return new Promise((resolve, reject) => {
    const check = (): void => {
      try {
        if (!exists(pid)) {
          resolve(true);
        } else if (Date.now() >= deadline) {
          resolve(false);
        } else {
          setTimeout(check, 25);
        }
      } catch {
        reject(new Error("Owned process cleanup failed"));
      }
    };
    check();
  });
}

async function bounded<T>(
  promise: Promise<T>,
  timeout: number,
  message: string,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | null = null;
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(message)), timeout);
      }),
    ]);
  } finally {
    if (timer) {
      clearTimeout(timer);
    }
  }
}

export class ProcessOwner {
  private children: OwnedChild[] = [];
  constructor(private readonly manifest: string) {}

  private record(event: string, child: OwnedChild, error?: unknown): void {
    appendFileSync(
      this.manifest,
      JSON.stringify({
        event,
        pid: child.child.pid ?? null,
        role: child.role,
        label: child.label,
        failure:
          error instanceof Error ? diagnosticTail(`${error.message}\n`) : null,
      }) + "\n",
      { mode: 0o600 },
    );
  }
  start(
    label: string,
    command: string,
    args: string[],
    cwd: string,
    env: NodeJS.ProcessEnv,
    role: "command" | "service" = "service",
  ): OwnedChild {
    const child = new OwnedChild(label, command, args, cwd, env, role);
    this.children.push(child);
    this.record("started", child);
    return child;
  }
  async run(
    label: string,
    command: string,
    args: string[],
    cwd: string,
    env: NodeJS.ProcessEnv,
    timeout: number,
  ): Promise<void> {
    await this.start(label, command, args, cwd, env, "command").finished(
      timeout,
    );
  }
  async stopRunning(): Promise<void> {
    const failures: unknown[] = [];
    for (const child of [...this.children].reverse()) {
      if (child.running) {
        try {
          await child.stop();
          this.record("stopped", child);
        } catch (error) {
          failures.push(error);
          this.record("cleanup_failed", child, error);
        }
      }
    }
    if (failures.length) {
      throw new AggregateError(
        failures,
        "An owned running child failed cleanup",
      );
    }
  }
  async stopAll(): Promise<void> {
    const failures: unknown[] = [];
    for (const child of [...this.children].reverse()) {
      try {
        await child.stop();
        this.record("stopped", child);
      } catch (error) {
        failures.push(error);
        this.record("cleanup_failed", child, error);
      }
    }
    this.children = [];
    if (failures.length) {
      throw new AggregateError(
        failures,
        "One or more owned process groups failed cleanup",
      );
    }
  }
}
