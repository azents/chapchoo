import assert from "node:assert/strict";
import test from "node:test";
import { InstructionSourceKind } from "@chapchoo/rpc-client/conversations";
import {
  instructionEvidence,
  instructionHistory,
} from "../dist/instruction-evidence.js";
const expected = {
  agentId: "agent",
  conversationId: "conversation",
  operationId: "operation",
};
const value = {
  ...expected,
  executionId: "execution",
  createdAt: 1000n,
  appliedAt: undefined,
  appliedTarget: undefined,
  sources: [
    {
      kind: InstructionSourceKind.INSTRUCTION_SOURCE_KIND_OWNER,
      text: "  Exact source\n\n",
    },
  ],
  renderedDocument: "# Owner instructions\n\n  Exact source\n\n",
  toolCatalog: Buffer.from("private tool contract"),
};
test("instruction evidence preserves exact sources and does not expose the operational tool catalogue", () => {
  const result = instructionEvidence(value, expected);
  assert.equal(result.sources[0].text, "  Exact source\n\n");
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
test("instruction evidence rejects mismatched attribution and incomplete application facts", () => {
  for (const change of [
    { agentId: "other" },
    { conversationId: "other" },
    { operationId: "other" },
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
  ])
    assert.throws(() => instructionEvidence({ ...value, ...change }, expected));
});
test("instruction history stays bounded, distinguishes prepared from supplied and preserves independent operations", () => {
  const item = {
    operationId: "one",
    executionId: "execution",
    createdAt: 1n,
    appliedAt: undefined,
  };
  assert.deepEqual(
    instructionHistory({
      items: [item, { ...item, operationId: "two", appliedAt: 2n }],
    }),
    {
      items: [
        {
          operationId: "one",
          executionId: "execution",
          createdAt: 1,
          appliedAt: undefined,
        },
        {
          operationId: "two",
          executionId: "execution",
          createdAt: 1,
          appliedAt: 2,
        },
      ],
      nextBefore: undefined,
    },
  );
  assert.throws(() => instructionHistory({ items: [item, item] }));
  assert.throws(() =>
    instructionHistory({
      items: Array.from({ length: 101 }, (_, index) => ({
        ...item,
        operationId: String(index),
      })),
    }),
  );
  assert.throws(() =>
    instructionHistory({
      items: [{ ...item, createdAt: BigInt(Number.MAX_SAFE_INTEGER) + 1n }],
    }),
  );
});
test("instruction history validates its bounded continuation cursor", () => {
  const item = {
    operationId: "tail",
    executionId: "execution",
    createdAt: 1n,
    appliedAt: undefined,
  };
  assert.equal(
    instructionHistory({ items: [item], nextBefore: "tail" }).nextBefore,
    "tail",
  );
  for (const nextBefore of ["", "other", "x".repeat(129)]) {
    assert.throws(() => instructionHistory({ items: [item], nextBefore }));
  }
  assert.throws(() => instructionHistory({ items: [], nextBefore: "tail" }));
  assert.deepEqual(instructionHistory({ items: [] }), {
    items: [],
    nextBefore: undefined,
  });
});
