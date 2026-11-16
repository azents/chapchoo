import "server-only";
import { instructionEvidence } from "@chapchoo/web-core/instruction-evidence";
import { call } from "@chapchoo/web-core/rpc";
import { getConfig } from "@/config";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import { loadAgents } from "../../server/loadAgents";
import type { InstructionEvidenceInput } from "../instructionEvidenceTypes";

export async function loadSystemPrompt(
  agentHandle: string,
  teamHandle: string,
): Promise<Omit<InstructionEvidenceInput, "basePath">> {
  const initial = await loadAgents(agentHandle, "", teamHandle);
  if (initial.type !== "detail") {
    throw new Error("System prompt requires an admitted Agent");
  }
  const scope = await readPageScope();
  const backend = getBackend(getConfig());
  const agentId = initial.agent.id;
  const value = await scope.read((token) =>
    call(
      backend.conversations.systemPrompt.bind(backend.conversations),
      { agentId },
      token,
    ),
  );
  return {
    teamHandle,
    agentHandle,
    identity: initial.identity,
    prompt: value.prompt
      ? instructionEvidence(value.prompt, { agentId })
      : null,
  };
}
