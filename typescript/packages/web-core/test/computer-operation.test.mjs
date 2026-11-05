import assert from "node:assert/strict";
import test from "node:test";
import { computerCopy, computerOperation } from "../dist/computer-messages.js";

test("operation copy distinguishes shutdown, following target and unverified completion in every locale", () => {
  for (const locale of ["en-US", "ko-KR", "ja-JP", "fr-FR"]) {
    const copy = computerCopy(locale);
    assert.deepEqual(
      computerOperation(
        { operation: "stop", intent: "running", phase: "blocked" },
        copy,
      ),
      { label: copy.blocked, hint: copy.blockedHint },
    );
    assert.deepEqual(
      computerOperation(
        {
          operation: "stop",
          intent: "stopped",
          phase: "uncertain",
        },
        copy,
      ),
      { label: copy.checkingStop, hint: null },
    );
    assert.equal(
      computerOperation(
        {
          operation: "stop",
          intent: "running",
          phase: "retiring",
        },
        copy,
      ).label,
      copy.startAfterStop,
    );
    assert.deepEqual(
      computerOperation(
        { operation: "stop", intent: "running", phase: "uncertain" },
        copy,
      ),
      { label: copy.startAfterStop, hint: null },
    );
    assert.deepEqual(
      computerOperation(
        {
          operation: "start",
          intent: "running",
          phase: "uncertain",
        },
        copy,
      ),
      { label: copy.checkingStart, hint: copy.checkingStartHint },
    );
    assert.deepEqual(
      computerOperation(
        { operation: "restart", intent: "running", phase: "uncertain" },
        copy,
      ),
      { label: copy.checkingRestart, hint: null },
    );
    assert.deepEqual(
      computerOperation(
        {
          operation: "delete",
          intent: "deleting",
          phase: "uncertain",
        },
        copy,
      ),
      { label: copy.checkingDelete, hint: copy.checkingDeleteHint },
    );
  }
});

test("Korean restart progress uses a concise task label", () => {
  const copy = computerCopy("ko-KR");
  for (const operation of ["stop", "restart"]) {
    assert.deepEqual(
      computerOperation(
        { operation, intent: "running", phase: "uncertain" },
        copy,
      ),
      { label: "재시작 중", hint: null },
    );
  }
});
