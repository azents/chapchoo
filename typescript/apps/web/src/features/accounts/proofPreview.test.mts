import assert from "node:assert/strict";
import test from "node:test";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { readProofPreview } from "./proofPreview.ts";

void test("malformed and unusable proof reads preserve the invalid-link outcome", async () => {
  const unusable = { usable: false, expiresAt: void 0 };
  assert.deepEqual(
    await readProofPreview(() =>
      Promise.reject(
        new RpcFailure(3 /* INVALID_ARGUMENT */, "DOMAIN_REASON_INVALID_PROOF"),
      ),
    ),
    unusable,
  );
  assert.deepEqual(
    await readProofPreview(() => Promise.resolve(unusable)),
    unusable,
  );
});
void test("proof infrastructure failures remain route failures without replay", async () => {
  const error = new RpcFailure(14 /* UNAVAILABLE */, null);
  let calls = 0;
  await assert.rejects(
    readProofPreview(() => {
      calls++;
      return Promise.reject(error);
    }),
    (actual) => actual === error,
  );
  assert.equal(calls, 1);
});
