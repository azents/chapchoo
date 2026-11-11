import type { ConversationAnchor } from "@chapchoo/web-core/conversation-types";

export interface RecordArrival {
  source: string;
  tail: ConversationAnchor | null;
}

/** Ordinal/index ordering distinguishes a new event from updates and prepends. */
export function hasRecordArrival(
  previous: RecordArrival,
  current: RecordArrival,
): boolean {
  if (previous.source !== current.source || !current.tail) {
    return false;
  }
  return (
    !previous.tail ||
    current.tail.ordinal > previous.tail.ordinal ||
    (current.tail.ordinal === previous.tail.ordinal &&
      current.tail.index > previous.tail.index)
  );
}
