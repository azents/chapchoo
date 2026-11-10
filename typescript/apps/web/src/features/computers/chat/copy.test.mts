import assert from "node:assert/strict";
import { test } from "node:test";
import {
  activityStatus,
  chatCopy,
  inputStatus,
  preparationStatus,
} from "./copy.ts";

void test("startup diagnoses are localized, harness-specific and distinct from actual read errors", () => {
  const cases: [string, string][] = [
    ["ko-KR", "Codex 실행 실패"],
    ["en-US", "Codex failed to start"],
    ["ja-JP", "Codexの起動に失敗しました"],
    ["fr-FR", "Échec du démarrage de Codex"],
  ];
  for (const [locale, expected] of cases) {
    const copy = chatCopy(locale);
    assert.equal(activityStatus(copy, "startFailed"), expected);
    assert.equal(
      preparationStatus(copy, { type: "failed", reason: "nativeRejected" }),
      expected,
    );
    assert.equal(
      activityStatus({ ...copy, codex: "Harness" }, "startFailed").includes(
        "Harness",
      ),
      true,
    );
    assert.notEqual(copy.error, expected);
    assert.notEqual(activityStatus(copy, "starting"), expected);
    assert.notEqual(activityStatus(copy, "authenticationRequired"), expected);
    assert.equal(preparationStatus(copy, { type: "ready" }), null);
    assert.equal(preparationStatus(copy), null);
    assert.equal(
      inputStatus(copy, {
        observation: "uncertain",
        preparation: { type: "preparing" },
      }),
      copy.uncertain,
    );
  }
});
