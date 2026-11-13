import type {
  ConversationMessage,
  ConversationMessageRegion,
} from "./conversation-message-types.js";

function sameMessage(
  left: ConversationMessage,
  right: ConversationMessage,
): boolean {
  return (
    left.id === right.id &&
    left.role === right.role &&
    left.order === right.order &&
    left.receivedAt === right.receivedAt &&
    left.text === right.text &&
    left.sourceInputId === right.sourceInputId &&
    left.sourceInputRequestId === right.sourceInputRequestId &&
    left.originalAcceptedAt === right.originalAcceptedAt
  );
}
/** Receipt identity/order is independent of native epochs and text equality. */
export function mergeConversationMessages(
  previous: ConversationMessageRegion,
  incoming: ConversationMessageRegion,
): ConversationMessageRegion {
  if (
    previous.agentId !== incoming.agentId ||
    previous.conversationId !== incoming.conversationId
  )
    return incoming;
  const messages = new Map(
    previous.messages.map((message) => [message.id, message]),
  );
  for (const message of incoming.messages) {
    const existing = messages.get(message.id);
    if (existing && !sameMessage(existing, message))
      throw new Error("Immutable conversation message changed");
    messages.set(message.id, message);
  }
  const ordered = [...messages.values()].sort((left, right) =>
    left.order < right.order ? -1 : left.order > right.order ? 1 : 0,
  );
  if (
    ordered.some(
      (message, index) =>
        index > 0 && message.order === ordered[index - 1]?.order,
    )
  )
    throw new Error("Conversation receipt order conflicts");
  // An explicit latest-window reset must not preserve a disconnected old
  // prefix whose next page could no longer reach the intervening receipts.
  if (incoming.window.type === "latest") return incoming;
  const isOlder = incoming.window.type === "before";
  const previousFirst = previous.messages[0]?.order;
  const incomingFirst = incoming.messages[0]?.order;
  return {
    ...incoming,
    messages: ordered,
    through: isOlder ? previous.through : incoming.through,
    next: isOlder ? previous.next : incoming.next,
    older: isOlder
      ? incoming.older
      : previousFirst !== undefined &&
          (incomingFirst === undefined || previousFirst <= incomingFirst)
        ? previous.older
        : incoming.older,
  };
}
