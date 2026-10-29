import "server-only";
import { createBackend } from "@chapchoo/web-core/rpc";
import type { WebConfig } from "@chapchoo/web-core/config";
import type { BackendClients } from "@chapchoo/web-core/rpc";

// Channels share immutable capabilities only. Credentials live in each call's metadata.
const channels = new Map<string, BackendClients>();
export function getBackend(config: WebConfig): BackendClients {
  const existing = channels.get(config.backendUrl);
  if (existing) {
    return existing;
  }
  const backend = createBackend(config);
  channels.set(config.backendUrl, backend);
  return backend;
}
