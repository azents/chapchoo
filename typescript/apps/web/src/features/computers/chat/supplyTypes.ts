import type { SupplyCopy } from "./supplyCopy";
import type {
  ConversationHeader,
  HarnessSupplyChoice,
  SavedHarnessSupply,
} from "@chapchoo/web-core/conversation-types";

export interface SupplyInput {
  agentId: string;
  unavailable: boolean;
  applied?: HarnessSupplyChoice;
  observation?: Pick<
    ConversationHeader,
    "preparation" | "savedAuthenticationId" | "appliedAuthenticationId"
  >;
}
export type SupplyState =
  | { type: "loading" }
  | { type: "error" }
  | {
      type: "ready";
      saved: SavedHarnessSupply;
      applied?: HarnessSupplyChoice;
      draft: HarnessSupplyChoice;
      submission: "ready" | "saving" | "saved" | "uncertain" | "rejected";
      stale: boolean;
      writable: boolean;
      saveable: boolean;
    };
export interface SupplyProps {
  observation?: SupplyInput["observation"];
  copy: SupplyCopy;
  opened: boolean;
  state: SupplyState;
  onToggle: () => void;
  onMode: (mode: "managed" | "existing") => void;
  onPath: (path: string) => void;
  onSave: () => void;
  onRead: () => void;
}
