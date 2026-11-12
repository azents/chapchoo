import type { ConversationAnchor } from "@chapchoo/web-core/conversation-types";

export interface MessageArrival {
  source: string;
  tail: bigint | null;
}
/** Receipt ordering distinguishes new messages from updates and prepends. */
export function hasMessageArrival(
  previous: MessageArrival,
  current: MessageArrival,
): boolean {
  return (
    previous.source === current.source &&
    current.tail !== null &&
    (previous.tail === null || current.tail > previous.tail)
  );
}
export interface RecordArrival {
  source: string;
  tail: ConversationAnchor | null;
}
/** Captured record ordering remains independent of message receipt ordering. */
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
