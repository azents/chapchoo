"use client";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useLocale } from "next-intl";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { instructionEvidenceCopy } from "../instructionEvidenceCopy";
import type {
  InstructionEvidenceInput,
  InstructionEvidenceProps,
} from "../instructionEvidenceTypes";

export function useInstructionEvidence(
  input: InstructionEvidenceInput,
): InstructionEvidenceProps {
  const locale = useLocale();
  const common = computerCopy(locale);
  return {
    ...input,
    copy: instructionEvidenceCopy(locale),
    agentsLabel: common.agents,
    replayLabel: common.replay,
    appearance: useAppearanceControl(),
  };
}
