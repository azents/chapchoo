import { useState } from "react";
import type { OwnerAction } from "@chapchoo/web-core/computer";

type Action = OwnerAction | "force";
interface Confirmation {
  action: Exclude<Action, "start">;
  requestId: string;
}
interface ComputerActions {
  confirmation: Confirmation | null;
  select: (action: Action, requestId: string) => void;
  confirm: () => void;
  cancel: () => void;
  complete: () => void;
}

/** One consent boundary shared by the real container and interaction fixtures. */
export function useComputerActions({
  pending,
  submit,
  reset,
}: {
  pending: boolean;
  submit: (action: Action, requestId: string) => void;
  reset: () => void;
}): ComputerActions {
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null);
  return {
    confirmation,
    select: (action, requestId) => {
      if (pending || confirmation) {
        return;
      }
      if (action === "start") {
        submit(action, requestId);
      } else {
        reset();
        setConfirmation({ action, requestId });
      }
    },
    confirm: () => {
      if (confirmation && !pending) {
        submit(confirmation.action, confirmation.requestId);
      }
    },
    cancel: () => {
      if (!pending) {
        setConfirmation(null);
      }
    },
    complete: () => setConfirmation(null),
  };
}
