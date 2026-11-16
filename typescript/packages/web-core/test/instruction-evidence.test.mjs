import assert from "node:assert/strict";
import test from "node:test";
import { InstructionSourceKind } from "@chapchoo/rpc-client/conversations";
import { instructionEvidence } from "../dist/instruction-evidence.js";
const expected = { agentId: "agent" };
const value = {
  ...expected,
  conversationId: "conversation",
  operationId: "operation",
  executionId: "execution",
  createdAt: 1000n,
  appliedAt: undefined,
  appliedTarget: undefined,
  sources: [
    {
      kind: InstructionSourceKind.INSTRUCTION_SOURCE_KIND_OWNER,
      text: "  Exact instruction\n\n",
    },
  ],
  renderedDocument: "# Owner instructions\n\n  Exact instruction\n\n",
  toolCatalog: Buffer.from("private tool contract"),
};
test("current prompt preserves exact instructions without exposing the operational tool catalogue", () => {
  const result = instructionEvidence(value, expected);
  assert.equal(result.sources[0].text, "  Exact instruction\n\n");
  assert.equal(result.appliedAt, undefined);
  assert.equal("toolCatalog" in result, false);
  assert.deepEqual(
    instructionEvidence({ ...value, sources: [] }, expected).sources,
    [],
  );
  assert.equal(
    instructionEvidence(
      {
        ...value,
        appliedAt: 1001n,
        appliedTarget: { snapshotId: "snapshot", nativeSessionId: "native" },
      },
      expected,
    ).appliedAt,
    1001,
  );
});
test("current prompt rejects wrong owner attribution, malformed identities and incomplete application facts", () => {
  for (const change of [
    { agentId: "other" },
    { conversationId: "" },
    { operationId: "" },
    { executionId: "" },
    { appliedAt: 1001n },
    { appliedTarget: { snapshotId: "snapshot", nativeSessionId: "native" } },
    {
      appliedAt: 1001n,
      appliedTarget: { snapshotId: "", nativeSessionId: "native" },
    },
    {
      sources: [
        {
          kind: InstructionSourceKind.INSTRUCTION_SOURCE_KIND_UNSPECIFIED,
          text: "unknown",
        },
      ],
    },
    { renderedDocument: "x".repeat(32 * 1024 + 1) },
    { createdAt: BigInt(Number.MAX_SAFE_INTEGER) + 1n },
  ])
    assert.throws(() => instructionEvidence({ ...value, ...change }, expected));
});
