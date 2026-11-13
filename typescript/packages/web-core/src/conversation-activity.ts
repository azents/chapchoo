import type { ConversationRegion } from "./conversation-types.js";

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
): ConversationActivity {
  const header = region.header;
  if (header.foreground !== "unavailable") return header.foreground;
  if (header.state === "notStarted" && !header.savedAuthenticationId)
    return "authenticationRequired";
  const latest = region.latestInput;
  const preparation =
    header.startup?.type === "failed" || header.startup?.type === "preparing"
      ? header.startup
      : latest?.preparation;
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
  const observation = latest?.observation;
  if (observation === "preparing" || observation === "accepted")
    return "starting";
  if (observation === "uncertain") return "deliveryUnconfirmed";
  if (observation === "failed") return "deliveryFailed";
  return header.state === "notStarted" ? "notStarted" : "unavailable";
}
