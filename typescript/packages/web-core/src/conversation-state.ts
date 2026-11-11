import type {
  ConversationRegion,
  LocalConversationInput,
} from "./conversation-types.js";
/** Persisted request identity supersedes local uncertainty without dispatching another operation. */
export function reconcileLocalConversationInputs(
  local: LocalConversationInput[],
  retained: ConversationRegion["inputs"],
  closed: string[] = [],
): LocalConversationInput[] {
  const remaining = local.filter(
    (item) =>
      !(item.id && closed.includes(item.id)) &&
      !retained?.some((input) => input.requestId === item.requestId),
  );
  return remaining.length === local.length ? local : remaining;
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
    (previous.inputs ?? [])
      .filter((input) => !incoming.closedInputs?.includes(input.id))
      .map((input) => [input.id, input]),
  );
  for (const input of incoming.inputs ?? []) {
    if (incoming.closedInputs?.includes(input.id)) continue;
    const old = inputs.get(input.id);
    const deleted = input.deleted || old?.deleted;
    inputs.set(input.id, {
      ...input,
      deleted,
      text: deleted ? undefined : (input.text ?? old?.text),
    });
  }
  return {
    ...incoming,
    latestInput: [...inputs.values()].some(
      (item) =>
        item.deleted && item.requestId === incoming.latestInput?.requestId,
    )
      ? undefined
      : incoming.latestInput,
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
/** Only acknowledged deletion or an exact tombstone read may call this. */
export function markConversationInputDeleted(
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
  const deleted = region.inputs?.find((item) => item.id === id);
  return {
    ...region,
    inputs: region.inputs?.map((input) =>
      input.id === id ? { ...input, deleted: true, text: undefined } : input,
    ),
    latestInput:
      region.latestInput?.requestId === deleted?.requestId
        ? undefined
        : region.latestInput,
  };
}
