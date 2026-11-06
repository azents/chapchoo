import assert from "node:assert/strict";
import test from "node:test";
import {
  authentication,
  authenticationSelection,
} from "../dist/external-authentication.js";
const now = { seconds: 1n, nanos: 0 };
const record = {
  id: "credential",
  scope: { personal: {} },
  kind: 1,
  name: "Account",
  currentRequestId: "registered",
  createdAt: now,
  updatedAt: now,
};
void test("catalogue retains accepted absent identity and complete Unicode alias bounds", () => {
  const noMetadata = authentication(record);
  assert.equal(noMetadata.accountId, null);
  assert.equal(noMetadata.email, null);
  assert.equal(noMetadata.planType, null);
  const name = "😀".repeat(256);
  assert.equal(authentication({ ...record, name }).name, name);
  assert.equal(
    authenticationSelection({
      authenticationId: record.id,
      name,
      kind: 1,
      requestId: "selected",
      selectedAt: now,
    }).name,
    name,
  );
  assert.throws(() => authentication({ ...record, name: "A".repeat(257) }));
  assert.throws(() =>
    authenticationSelection({
      authenticationId: record.id,
      name: "A".repeat(257),
      kind: 1,
      requestId: "selected",
      selectedAt: now,
    }),
  );
});
void test("optional provider metadata admits the backend byte bound without inventing missing values", () => {
  const value = "😀".repeat(1024);
  const saved = authentication({
    ...record,
    accountId: value,
    email: value,
    planType: value,
  });
  assert.equal(saved.accountId, value);
  assert.equal(saved.email, value);
  assert.equal(saved.planType, value);
  assert.throws(() => authentication({ ...record, accountId: `${value}A` }));
  assert.throws(() => authentication({ ...record, email: "bad\nemail" }));
});
