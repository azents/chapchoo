import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import en from "../../../messages/en-US.json" with { type: "json" };
import fr from "../../../messages/fr-FR.json" with { type: "json" };
import ja from "../../../messages/ja-JP.json" with { type: "json" };
import ko from "../../../messages/ko-KR.json" with { type: "json" };
import { accountError } from "./errors.ts";

void test("collaborative context badges use the approved localized Team term", () => {
  for (const [locale, expected] of [
    [en, "Team"],
    [ko, "팀"],
    [ja, "チーム"],
    [fr, "Équipe"],
  ] as const) {
    assert.equal(locale.accounts.team, expected);
    assert.equal("workspace" in locale.accounts, false);
  }
  const component = readFileSync(
    new URL("./components/Workspace.tsx", import.meta.url),
    "utf8",
  );
  assert.match(
    component,
    /contextLabel=\{team\?\.personal \? c\.personal : c\.team\}/,
  );
});

void test("all product locales have the same complete message keys", () => {
  const expected = Object.keys(en.accounts).sort();
  for (const locale of [en, fr, ja, ko]) {
    assert.deepEqual(Object.keys(locale.accounts).sort(), expected);
    assert.ok(
      Object.values(locale.accounts).every((value) => value.trim().length > 0),
    );
  }
});

void test("domain failures are localized without exposing machine reasons", () => {
  for (const locale of [en, fr, ja, ko]) {
    const c = locale.accounts;
    assert.equal(
      accountError("DOMAIN_REASON_INVALID_SESSION", c),
      c.invalidSession,
    );
    assert.equal(
      accountError("DOMAIN_REASON_PERMISSION_DENIED", c),
      c.permissionDenied,
    );
    assert.equal(
      accountError("DOMAIN_REASON_MAIL_UNAVAILABLE", c),
      c.mailUncertain,
    );
    assert.equal(
      accountError("DOMAIN_REASON_OWNER_TRANSFER_REQUIRED", c),
      c.ownerLeaveDescription,
    );
    assert.equal(
      accountError("DOMAIN_REASON_INVALID_PROOF", c),
      c.invalidProof,
    );
    assert.equal(
      accountError("DOMAIN_REASON_EMAIL_MISMATCH", c),
      c.emailMismatch,
    );
    assert.equal(accountError("UNKNOWN_REASON", c), c.genericError);
    assert.equal(accountError(null, c), c.genericError);
  }
});
