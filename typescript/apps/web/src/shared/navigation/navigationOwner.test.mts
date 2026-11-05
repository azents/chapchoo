import assert from "node:assert/strict";
import test from "node:test";
import { navigationOwnerMatches } from "./navigationOwner.ts";
import type { InitialNavigation } from "./types";

void test("private navigation hides immediately on identity withdrawal or replacement, before cache-clearing effects", () => {
  const admitted: InitialNavigation = {
    type: "ready",
    userId: "account-a",
    teams: [],
    preferred: null,
    renderedAt: 0,
  };
  assert.equal(navigationOwnerMatches(admitted, "account-a"), true);
  assert.equal(navigationOwnerMatches(admitted, "account-b"), false);
  assert.equal(navigationOwnerMatches(admitted, null), false);
  assert.equal(navigationOwnerMatches({ type: "guest" }, "account-a"), false);
  assert.equal(
    navigationOwnerMatches({ type: "error", userId: "account-a" }, "account-a"),
    true,
  );
  assert.equal(
    navigationOwnerMatches({ type: "error", userId: "account-a" }, "account-b"),
    false,
  );
});
