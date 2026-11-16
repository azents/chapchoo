"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { InstructionEvidence } from "./components/InstructionEvidence";
import { useInstructionEvidence } from "./containers/useInstructionEvidence";
export const InstructionEvidencePage = createReactContainer(
  "InstructionEvidencePage",
  useInstructionEvidence,
  InstructionEvidence,
);
