import { z } from "zod";

const requestId = z.uuid();
const id = z.string().min(1).max(128);
const schema = z.discriminatedUnion("action", [
  z.object({ requestId, action: z.literal("create") }).strict(),
  z.object({ requestId, action: z.literal("update"), id }).strict(),
  z.object({ requestId, action: z.literal("delete"), id }).strict(),
]);
export type PendingMcpWrite = z.infer<typeof schema>;
const changed = "chapchoo:mcp-write";

export function pendingWriteKey(owner: string, agent: string): string {
  return `${changed}:${encodeURIComponent(owner)}:${encodeURIComponent(agent)}`;
}
export function readPendingWrite(key: string | null): string | null {
  try {
    return key === null ? null : window.sessionStorage.getItem(key);
  } catch {
    return null;
  }
}
export function decodePendingWrite(raw: string | null): PendingMcpWrite | null {
  if (!raw || raw.length > 512) {
    return null;
  }
  try {
    const value = schema.safeParse(JSON.parse(raw));
    return value.success ? value.data : null;
  } catch {
    return null;
  }
}
export function storePendingWrite(
  key: string,
  write: PendingMcpWrite | null,
): boolean {
  try {
    if (write) {
      window.sessionStorage.setItem(key, JSON.stringify(schema.parse(write)));
    } else {
      window.sessionStorage.removeItem(key);
    }
    window.dispatchEvent(new Event(changed));
    return true;
  } catch {
    return false;
  }
}
export function subscribePendingWrite(listener: () => void): () => void {
  window.addEventListener(changed, listener);
  return () => window.removeEventListener(changed, listener);
}
