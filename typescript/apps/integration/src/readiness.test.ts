import assert from "node:assert/strict";
import test from "node:test";
import {
  Computer,
  Connectivity,
  Phase,
  Power,
} from "@chapchoo/rpc-client/computers";
import { lifecycleReady, poll } from "./readiness.ts";

function computer(phase: Phase): Computer {
  return Computer.fromPartial({
    requestId: "stop",
    phase,
    power: Power.POWER_STOPPED,
    connector: Connectivity.CONNECTIVITY_DISCONNECTED,
  });
}

void test("native readiness waits for the same request's authoritative completion", () => {
  assert.deepEqual(
    [Phase.PHASE_RETIRING, Phase.PHASE_UNCERTAIN, Phase.PHASE_COMPLETED].map(
      (phase) =>
        lifecycleReady(computer(phase), "stop", Power.POWER_STOPPED, false),
    ),
    [false, false, true],
  );
  assert.equal(
    lifecycleReady(
      computer(Phase.PHASE_COMPLETED),
      "stop",
      Power.POWER_RUNNING,
      true,
    ),
    false,
  );
  assert.throws(
    () =>
      lifecycleReady(
        computer(Phase.PHASE_COMPLETED),
        "restart",
        Power.POWER_STOPPED,
        false,
      ),
    /correlation changed/,
  );
  assert.throws(
    () =>
      lifecycleReady(
        computer(Phase.PHASE_BLOCKED),
        "stop",
        Power.POWER_STOPPED,
        false,
      ),
    /blocked/,
  );
});

void test("persistent native uncertainty exhausts the finite read-only budget", async () => {
  let reads = 0;
  await assert.rejects(
    poll(
      () => {
        reads += 1;
        return Promise.resolve(computer(Phase.PHASE_UNCERTAIN));
      },
      (value) => lifecycleReady(value, "stop", Power.POWER_STOPPED, false),
      new AbortController().signal,
      5,
    ),
    /readiness deadline exceeded/,
  );
  assert.ok(reads > 0);
});
