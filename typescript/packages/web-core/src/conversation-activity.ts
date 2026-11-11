import type {
  ConversationRegion,
  LocalConversationInput,
} from "./conversation-types.js";

export type ConversationActivity =
  | "idle"
  | "working"
  | "requiresAction"
  | "notStarted"
  | "authenticationRequired"
  | "starting"
  | "startFailed"
  | "sourceUnavailable"
  | "retirementUnknown"
  | "deliveryFailed"
  | "deliveryUnconfirmed"
  | "unavailable";

/** Project retained facts, never absence alone, into a failed-start diagnosis. */
export function conversationActivity(
  region: ConversationRegion,
  local: LocalConversationInput[] = [],
): ConversationActivity {
  const header = region.header;
  if (header.foreground !== "unavailable") return header.foreground;
  const latestLocal = local
    .filter(
      (item) =>
        item.requestId !== region.latestInput?.requestId &&
        (item.conversationId === undefined ||
          item.conversationId === header.conversationId) &&
        !region.inputs?.some(
          (retained) => retained.requestId === item.requestId,
        ) &&
        (item.snapshotId === undefined ||
          item.snapshotId === header.source?.snapshotId),
    )
    .at(-1);
  if (
    latestLocal?.observation === "sending" ||
    latestLocal?.observation === "accepted" ||
    latestLocal?.observation === "preparing"
  )
    return "starting";
  if (latestLocal?.observation === "uncertain") return "deliveryUnconfirmed";
  if (header.state === "notStarted" && !header.savedAuthenticationId)
    return "authenticationRequired";
  const latest = region.latestInput;
  const preparation = latest?.preparation ?? header.startup;
  if (preparation?.type === "failed") {
    switch (preparation.reason) {
      case "sourceUnavailable":
        return "sourceUnavailable";
      case "retirementUnknown":
        return "retirementUnknown";
      default:
        return "startFailed";
    }
  }
  if (header.state === "loading" || preparation?.type === "preparing")
    return "starting";
  const observation = latestLocal?.observation ?? latest?.observation;
  if (observation === "preparing" || observation === "accepted")
    return "starting";
  if (observation === "uncertain") return "deliveryUnconfirmed";
  if (observation === "failed") return "deliveryFailed";
  return header.state === "notStarted" ? "notStarted" : "unavailable";
}
