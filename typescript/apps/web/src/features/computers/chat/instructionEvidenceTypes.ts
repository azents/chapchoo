import type { InstructionEvidenceCopy } from "./instructionEvidenceCopy";
import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type { InstructionEvidenceView } from "@chapchoo/web-core/instruction-evidence";

export interface InstructionEvidenceInput {
  basePath: string;
  teamHandle: string;
  agentHandle: string;
  identity: string;
  prompt: InstructionEvidenceView | null;
}
export interface InstructionEvidenceProps extends InstructionEvidenceInput {
  copy: InstructionEvidenceCopy;
  agentsLabel: string;
  replayLabel: string;
  appearance: ColorModeMenuProps;
}
