import type {
  ConversationRegion,
  LocalConversationInput,
} from "./conversation-types.js";
/** Persisted request identity supersedes local uncertainty without dispatching another operation. */
export function reconcileLocalConversationInputs(
  local: LocalConversationInput[],
  retained: ConversationRegion["inputs"],
): LocalConversationInput[] {
  return local.filter(
    (item) => !retained?.some((input) => input.requestId === item.requestId),
  );
}
/** Newer updates win; a late older-page result only contributes missing or newer records. */
export function mergeConversationRegion(
  previous: ConversationRegion,
  incoming: ConversationRegion,
): ConversationRegion {
  if (
    previous.header.agentId !== incoming.header.agentId ||
    previous.header.conversationId !== incoming.header.conversationId ||
    previous.header.source?.snapshotId !== incoming.header.source?.snapshotId ||
    incoming.header.state === "loading"
  )
    return incoming;
  const records = new Map(
    previous.records.map((record) => [record.key, record]),
  );
  for (const record of incoming.records) {
    const old = records.get(record.key);
    if (!old || record.updatedOrdinal >= old.updatedOrdinal)
      records.set(record.key, record);
  }
  const inputs = new Map(
    (previous.inputs ?? []).map((input) => [input.id, input]),
  );
  for (const input of incoming.inputs ?? []) {
    const old = inputs.get(input.id);
    inputs.set(input.id, { ...input, text: input.text ?? old?.text });
  }
  return {
    ...incoming,
    inputs: [...inputs.values()].sort((a, b) =>
      a.acceptedAt < b.acceptedAt
        ? -1
        : a.acceptedAt > b.acceptedAt
          ? 1
          : a.requestId.localeCompare(b.requestId),
    ),
    records: [...records.values()].sort((a, b) =>
      a.anchor.ordinal < b.anchor.ordinal
        ? -1
        : a.anchor.ordinal > b.anchor.ordinal
          ? 1
          : a.anchor.index < b.anchor.index
            ? -1
            : a.anchor.index > b.anchor.index
              ? 1
              : 0,
    ),
  };
}
