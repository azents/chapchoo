import assert from "node:assert/strict";
import test from "node:test";
import { activeLoginChallenge } from "./methodState.ts";

void test("history returning to password cannot inherit code-only field locking", () => {
  const challenge = { email: "admin@example.test", value: "fixture-binding" };
  assert.equal(activeLoginChallenge("code", challenge), challenge);
  assert.equal(activeLoginChallenge("password", challenge), null);
  assert.equal(activeLoginChallenge("code", null), null);
});
