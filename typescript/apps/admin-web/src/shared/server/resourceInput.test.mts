import assert from "node:assert/strict";
import test from "node:test";
import { resourceInput, resourceSelection } from "./resourceInput.ts";

void test("resource URL cursors ignore decoration and incomplete or repeated fields", () => {
  const first = { computerId: "computer" };
  for (const search of [
    {},
    { tracking: "unrelated" },
    { kind: "PVC" },
    { kind: "PVC", namespace: "managed" },
    { kind: ["PVC", "POD"], namespace: "managed", name: "original" },
    { kind: "PVC", namespace: ["managed"], name: "original" },
    { kind: "PVC", namespace: "managed", name: ["original"] },
    { kind: "OTHER", namespace: "managed", name: "original" },
    { kind: "PVC", namespace: "", name: "original" },
    { kind: "PVC", namespace: "managed", name: "x".repeat(254) },
  ]) {
    assert.deepEqual(resourceSelection("computer", search), first);
  }
  assert.deepEqual(
    resourceSelection("computer", {
      kind: "PVC",
      namespace: "managed",
      name: "original",
      tracking: "decoration",
    }),
    {
      computerId: "computer",
      after: { kind: "PVC", namespace: "managed", name: "original" },
    },
  );
});
void test("the RPC resource input remains strict even though browser URLs are normalized", () => {
  assert.equal(
    resourceInput.safeParse({
      computerId: "computer",
      after: { kind: "PVC", namespace: "managed" },
    }).success,
    false,
  );
  assert.equal(
    resourceInput.safeParse({ computerId: "computer", extra: "untrusted" })
      .success,
    false,
  );
  assert.throws(() => resourceSelection("", {}));
});
