import { InstructionSourceKind } from "@chapchoo/rpc-client/conversations";
import { invalidResponse } from "./errors.js";
import type {
  ProjectInstructionEvidence,
  ProjectInstructionHistory,
} from "@chapchoo/rpc-client/conversations";

export type InstructionSourceRole =
  "product" | "function" | "owner" | "execution";
export interface InstructionEvidenceSource {
  role: InstructionSourceRole;
  text: string;
}
export interface InstructionHistoryItem {
  operationId: string;
  executionId: string;
  createdAt: number;
  appliedAt?: number;
}
export interface InstructionHistoryPage {
  items: InstructionHistoryItem[];
  nextBefore?: string;
}
export interface InstructionEvidenceView extends InstructionHistoryItem {
  agentId: string;
  conversationId: string;
  sources: InstructionEvidenceSource[];
  document: string;
}
function identity(value: string): string {
  return value.length > 0 && value.length <= 128 ? value : invalidResponse();
}
function instant(value: bigint): number {
  return value >= 0n && value <= BigInt(Number.MAX_SAFE_INTEGER)
    ? Number(value)
    : invalidResponse();
}
function role(value: InstructionSourceKind): InstructionSourceRole {
  switch (value) {
    case InstructionSourceKind.INSTRUCTION_SOURCE_KIND_PRODUCT:
      return "product";
    case InstructionSourceKind.INSTRUCTION_SOURCE_KIND_FUNCTION:
      return "function";
    case InstructionSourceKind.INSTRUCTION_SOURCE_KIND_OWNER:
      return "owner";
    case InstructionSourceKind.INSTRUCTION_SOURCE_KIND_EXECUTION:
      return "execution";
    default:
      return invalidResponse();
  }
}
export function instructionHistory(
  value: ProjectInstructionHistory,
): InstructionHistoryPage {
  if (value.items.length > 100) return invalidResponse();
  const identities = new Set<string>();
  const items = value.items.map((item) => {
    const operationId = identity(item.operationId);
    if (identities.has(operationId)) return invalidResponse();
    identities.add(operationId);
    return {
      operationId,
      executionId: identity(item.executionId),
      createdAt: instant(item.createdAt),
      appliedAt:
        item.appliedAt === undefined ? undefined : instant(item.appliedAt),
    };
  });
  const nextBefore =
    value.nextBefore === undefined ? undefined : identity(value.nextBefore);
  if (nextBefore !== undefined && items.at(-1)?.operationId !== nextBefore)
    return invalidResponse();
  return { items, nextBefore };
}
export function instructionEvidence(
  value: ProjectInstructionEvidence,
  expected: { agentId: string; conversationId: string; operationId: string },
): InstructionEvidenceView {
  if (
    value.agentId !== expected.agentId ||
    value.conversationId !== expected.conversationId ||
    value.operationId !== expected.operationId ||
    value.sources.length > 32 ||
    new TextEncoder().encode(value.renderedDocument).length > 32 * 1024
  )
    return invalidResponse();
  const sources = value.sources.map((source) => {
    const sourceRole = role(source.kind);
    if (new TextEncoder().encode(source.text).length > 32 * 1024)
      return invalidResponse();
    return { role: sourceRole, text: source.text };
  });
  if ((value.appliedAt === undefined) !== (value.appliedTarget === undefined))
    return invalidResponse();
  if (
    value.appliedTarget &&
    (!value.appliedTarget.snapshotId || !value.appliedTarget.nativeSessionId)
  )
    return invalidResponse();
  return {
    agentId: identity(value.agentId),
    conversationId: identity(value.conversationId),
    operationId: identity(value.operationId),
    executionId: identity(value.executionId),
    createdAt: instant(value.createdAt),
    appliedAt:
      value.appliedAt === undefined ? undefined : instant(value.appliedAt),
    sources,
    document: value.renderedDocument,
  };
}
