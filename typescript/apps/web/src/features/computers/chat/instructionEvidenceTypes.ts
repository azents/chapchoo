import type { InstructionEvidenceCopy } from "./instructionEvidenceCopy";
import type { InstructionEvidenceView } from "@chapchoo/web-core/instruction-evidence";

export type InstructionEvidenceState =
  | { type: "loading" }
  | { type: "error" }
  | { type: "empty" }
  | {
      type: "ready";
      selected: string;
      options: { value: string; label: string }[];
      more: { available: boolean; read: "ready" | "loading" | "error" };
      evidence:
        | { type: "loading" }
        | { type: "error" }
        | { type: "ready"; value: InstructionEvidenceView };
    };
export interface InstructionEvidenceInput {
  agentId: string;
  conversationId: string;
}
export interface InstructionEvidenceProps {
  opened: boolean;
  copy: InstructionEvidenceCopy;
  state: InstructionEvidenceState;
  onToggle: () => void;
  onSelect: (operationId: string) => void;
  onRead: () => void;
  onMore: () => void;
}
