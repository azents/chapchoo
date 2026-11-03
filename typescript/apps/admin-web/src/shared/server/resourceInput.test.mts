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
      after: {
        kind: "PVC",
        namespace: "managed",
        name: "original",
        nativeScopeKind: "",
        nativeScope: "",
      },
    },
  );
});
void test("Docker cursors round-trip exact opaque scope and refuse mixed or unbounded identities", () => {
  const after = {
    kind: "DOCKER_VOLUME",
    namespace: "",
    name: "original",
    nativeScopeKind: "DOCKER_DAEMON",
    nativeScope: `daemon-${"x".repeat(600)}`,
  };
  const search = Object.fromEntries(new URLSearchParams(after));
  assert.deepEqual(resourceSelection("computer", search), {
    computerId: "computer",
    after,
  });
  for (const invalid of [
    { ...after, namespace: "managed" },
    { ...after, nativeScopeKind: "" },
    { ...after, nativeScope: "" },
    { ...after, nativeScope: "bad daemon" },
    { ...after, nativeScope: "é".repeat(513) },
    { ...after, nativeScope: [after.nativeScope] },
    { ...after, kind: "PVC", namespace: "managed" },
    { ...after, name: "x".repeat(256) },
  ]) {
    assert.deepEqual(resourceSelection("computer", invalid), {
      computerId: "computer",
    });
    assert.equal(
      resourceInput.safeParse({ computerId: "computer", after: invalid })
        .success,
      false,
    );
  }
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
