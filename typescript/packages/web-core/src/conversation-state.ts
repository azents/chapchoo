import type { ConversationRegion } from "./conversation-types.js";
/** Newer updates win; a late older-page result only contributes missing or newer records. */
export function mergeConversationRegion(
  previous: ConversationRegion,
  incoming: ConversationRegion,
): ConversationRegion {
  if (
    previous.header.agentId !== incoming.header.agentId ||
    previous.header.conversationId !== incoming.header.conversationId ||
    previous.header.source?.snapshotId !== incoming.header.source?.snapshotId
  )
    return incoming;
  const records = new Map(
    (incoming.header.state === "loading" ? [] : previous.records).map(
      (record) => [record.key, record],
    ),
  );
  for (const record of incoming.records) {
    const old = records.get(record.key);
    if (!old || record.updatedOrdinal >= old.updatedOrdinal)
      records.set(record.key, record);
  }
  const inputs = new Map(
    (previous.inputs ?? [])
      .filter((input) => !incoming.closedInputs?.includes(input.id))
      .map((input) => [input.id, input]),
  );
  for (const input of incoming.inputs ?? []) {
    if (incoming.closedInputs?.includes(input.id)) continue;
    const old = inputs.get(input.id);
    inputs.set(input.id, {
      ...input,
      text: input.text ?? old?.text,
    });
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
/** Remove an acknowledged queue item under its exact queried source. */
export function removeConversationInput(
  region: ConversationRegion,
  id: string,
  conversationId: string,
  snapshotId?: string,
): ConversationRegion {
  if (
    region.header.conversationId !== conversationId ||
    region.header.source?.snapshotId !== snapshotId
  )
    return region;
  const removed = region.inputs?.find((item) => item.id === id);
  return {
    ...region,
    inputs: region.inputs?.filter((input) => input.id !== id),
    latestInput:
      region.latestInput?.requestId === removed?.requestId
        ? undefined
        : region.latestInput,
  };
}
