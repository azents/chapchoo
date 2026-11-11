import { computerOperation } from "@chapchoo/web-core/computer-messages";
import type { ComputerView } from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
import type { ConversationActivity } from "@chapchoo/web-core/conversation-activity";
import type { ConversationHeader } from "@chapchoo/web-core/conversation-types";

export interface ChatStatusView {
  label: string;
  refreshing: boolean;
}

/** Retained presentation never restores current power or native effect authority. */
export function chatStatus({
  computer,
  activity,
  foreground,
  read,
  stale,
  error,
  computerCopy,
  activityLabel,
}: {
  computer: ComputerView;
  activity: ConversationActivity;
  foreground: ConversationHeader["foreground"];
  read: "ready" | "loading" | "error";
  stale: boolean;
  error?: string | null;
  computerCopy: ComputerCopy;
  activityLabel: (value: ConversationActivity) => string;
}): ChatStatusView {
  if (error && !stale) {
    return { label: error, refreshing: false };
  }
  if (computer.phase !== "completed") {
    if (computer.phase !== "blocked" && computer.intent === "running") {
      if (computer.operation === "restart" || computer.phase === "retiring") {
        return { label: computerCopy.checkingRestart, refreshing: false };
      }
      if (computer.operation === "create" || computer.operation === "start") {
        return { label: computerCopy.starting, refreshing: false };
      }
    }
    if (computer.phase !== "blocked" && computer.operation === "stop") {
      return { label: computerCopy.retiring, refreshing: false };
    }
    return {
      label: computerOperation(computer, computerCopy).label,
      refreshing: false,
    };
  }
  if (computer.power === "stopped") {
    return { label: computerCopy.stopped, refreshing: false };
  }
  const refreshing =
    stale ||
    read !== "ready" ||
    computer.power === "unknown" ||
    computer.connector !== "connected";
  if (foreground !== "unavailable") {
    return { label: activityLabel(foreground), refreshing };
  }
  if (activity !== "unavailable") {
    const failure = [
      "startFailed",
      "sourceUnavailable",
      "retirementUnknown",
      "deliveryFailed",
      "deliveryUnconfirmed",
      "authenticationRequired",
    ].includes(activity);
    return {
      label: activityLabel(activity),
      refreshing: !failure && (refreshing || activity === "starting"),
    };
  }
  // Initial absence supplies no previous state to invent a label from.
  return { label: "", refreshing: true };
}
