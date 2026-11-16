import type { ReactElement } from "react";

export interface PresenceEntry {
  key: string;
  node: ReactElement;
  present: boolean;
  initial: boolean;
  order?: bigint;
}

/** Keep visual exits in their old position; prepended history is already visible. */
export function reconcilePresence(
  previous: readonly PresenceEntry[],
  next: readonly { key: string; node: ReactElement; order?: bigint }[],
): PresenceEntry[] {
  const old = new Map(previous.map((entry) => [entry.key, entry]));
  const oldest = previous.reduce<bigint | null>(
    (value, entry) =>
      entry.order !== void 0 && (value === null || entry.order < value)
        ? entry.order
        : value,
    null,
  );
  const result = next.map((entry) => ({
    ...entry,
    present: true,
    initial:
      old.get(entry.key)?.initial ??
      (oldest !== null && entry.order !== void 0 && entry.order < oldest),
  }));
  const keys = new Set(next.map((entry) => entry.key));
  previous.forEach((entry, index) => {
    if (!keys.has(entry.key)) {
      result.splice(Math.min(index, result.length), 0, {
        ...entry,
        present: false,
      });
    }
  });
  return result;
}
