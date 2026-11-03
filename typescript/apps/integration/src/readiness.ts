import { setTimeout as pause } from "node:timers/promises";
import { Connectivity, Phase } from "@chapchoo/rpc-client/computers";
import type { Computer, Power } from "@chapchoo/rpc-client/computers";

export function lifecycleReady(
  computer: Computer | undefined,
  requestId: string,
  expected: Power,
  connected: boolean,
): boolean {
  if (!computer || computer.requestId !== requestId) {
    throw new Error("Native lifecycle request correlation changed");
  }
  if (computer.phase === Phase.PHASE_BLOCKED) {
    throw new Error(
      "Native lifecycle is blocked; mutation will not be replayed",
    );
  }
  // UNCERTAIN also fences normally progressing once-only native retirement.
  // Only an authoritative completed report can satisfy this request.
  return (
    computer.phase === Phase.PHASE_COMPLETED &&
    computer.power === expected &&
    (!connected || computer.connector === Connectivity.CONNECTIVITY_CONNECTED)
  );
}

export async function poll<T>(
  read: () => Promise<T>,
  ready: (value: T) => boolean,
  signal: AbortSignal,
  timeoutMs = 90_000,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    signal.throwIfAborted();
    const value = await read();
    signal.throwIfAborted();
    if (ready(value)) {
      return value;
    }
    // Read-only authoritative polling never replays a lifecycle mutation.
    await pause(Math.min(250, Math.max(0, deadline - Date.now())), undefined, {
      signal,
    });
  }
  throw new Error(
    "Native integration authoritative readiness deadline exceeded",
  );
}
