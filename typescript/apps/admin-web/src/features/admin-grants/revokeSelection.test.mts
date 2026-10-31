import assert from "node:assert/strict";
import test from "node:test";
import { currentRevokeId } from "./revokeSelection.ts";

void test("history navigation cannot confirm a hidden grant from another inventory page", () => {
  const selection = { id: "fixture-grant", offset: 25n };
  assert.equal(currentRevokeId(selection, 25n), "fixture-grant");
  assert.equal(currentRevokeId(selection, 0n), null);
  assert.equal(currentRevokeId(selection, 50n), null);
  assert.equal(currentRevokeId(null, 25n), null);
});
