import assert from "node:assert/strict";
import test from "node:test";
import { nativeEvidence } from "./nativeEvidence.ts";

const fixture = {
  daemon: "daemon:fixture",
  engine: "29.3.0",
  api: "1.53",
  architecture: "amd64",
  image: `sha256:${"a".repeat(64)}`,
  original: "b".repeat(64),
  successor: "c".repeat(64),
  restarted: "d".repeat(64),
  volume: "saved-original",
  cleanup: "confirmed",
  canaries: "unchanged",
  source: "explicit-unix",
  network: "bridge",
};
void test("native evidence publishes only bounded safe facts from a successful lifecycle", () => {
  assert.deepEqual(
    nativeEvidence(
      JSON.stringify({ ...fixture, environment: "private-canary" }),
    ),
    fixture,
  );
  for (const invalid of [
    "not-json",
    JSON.stringify({ ...fixture, cleanup: "uncertain" }),
    JSON.stringify({ ...fixture, daemon: "secret\ncanary" }),
    JSON.stringify({ ...fixture, image: "tag:mutable" }),
    JSON.stringify({ ...fixture, source: "unexpected" }),
  ]) {
    assert.equal(nativeEvidence(invalid), null);
  }
});
