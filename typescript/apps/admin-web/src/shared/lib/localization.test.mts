import assert from "node:assert/strict";
import test from "node:test";
import en from "../../../messages/en-US.json" with { type: "json" };
import fr from "../../../messages/fr-FR.json" with { type: "json" };
import ja from "../../../messages/ja-JP.json" with { type: "json" };
import ko from "../../../messages/ko-KR.json" with { type: "json" };
import { errorKey } from "./errors.ts";
import { isSupportedLocale, SUPPORTED_LOCALES } from "./locale.ts";
import type { ErrorKey } from "./errors.ts";

interface MessageTree {
  readonly [key: string]: string | MessageTree;
}

const translations = { "ko-KR": ko, "ja-JP": ja, "fr-FR": fr };

function leaves(messages: MessageTree, prefix = ""): Map<string, string> {
  const result = new Map<string, string>();
  for (const [key, value] of Object.entries(messages)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (typeof value === "string") {
      result.set(path, value);
    } else {
      for (const [childPath, text] of leaves(value, path)) {
        result.set(childPath, text);
      }
    }
  }
  return result;
}

function placeholders(value: string): string[] {
  return Array.from(
    value.matchAll(/\{([A-Za-z][A-Za-z0-9_]*)\s*[,}]/g),
    (match) => match[1] ?? "",
  ).sort();
}

await test("Admin translations preserve all English namespaces and message keys", () => {
  const englishKeys = [...leaves(en).keys()].sort();
  for (const [locale, messages] of Object.entries(translations)) {
    assert.deepEqual(
      Object.keys(messages).sort(),
      Object.keys(en).sort(),
      locale,
    );
    assert.deepEqual([...leaves(messages).keys()].sort(), englishKeys, locale);
    for (const [path, value] of leaves(messages)) {
      assert.ok(value.trim().length > 0, `${locale}:${path}`);
      assert.ok(!value.includes("DOMAIN_REASON_"), `${locale}:${path}`);
    }
  }
});

await test("Admin translations retain each ICU placeholder and pagination variable", () => {
  const english = leaves(en);
  assert.deepEqual(placeholders(en.AdminGrants.range), ["from", "to", "total"]);
  for (const [locale, messages] of Object.entries(translations)) {
    for (const [path, value] of leaves(messages)) {
      const source = english.get(path);
      assert.ok(typeof source === "string", `${locale}:${path}`);
      assert.deepEqual(
        placeholders(value),
        placeholders(source),
        `${locale}:${path}`,
      );
    }
  }
});

await test("translated validity periods and password length preserve the English policy numbers", () => {
  for (const messages of Object.values(translations)) {
    assert.deepEqual(
      messages.AdminGrants.signupDescription.match(/\d+/g),
      en.AdminGrants.signupDescription.match(/\d+/g),
    );
    assert.deepEqual(
      messages.AdminGrants.resetDescription.match(/\d+/g),
      en.AdminGrants.resetDescription.match(/\d+/g),
    );
    assert.deepEqual(
      messages.AdminAuth.passwordPolicy.match(/\d+/g)?.slice(0, 2),
      en.AdminAuth.passwordPolicy.match(/\d+/g)?.slice(0, 2),
    );
  }
});

await test("known domain reasons resolve to translated presentation keys", () => {
  const known: ReadonlyArray<readonly [string, ErrorKey]> = [
    ["DOMAIN_REASON_INVALID_INPUT", "invalidInput"],
    ["DOMAIN_REASON_INVALID_CREDENTIALS", "invalidCredentials"],
    ["DOMAIN_REASON_INVALID_SESSION", "invalidSession"],
    ["DOMAIN_REASON_PERMISSION_DENIED", "permissionDenied"],
    ["DOMAIN_REASON_NOT_FOUND", "notFound"],
    ["DOMAIN_REASON_CONFLICT", "conflict"],
    ["DOMAIN_REASON_INVALID_TRANSITION", "invalidTransition"],
    ["DOMAIN_REASON_INVALID_PROOF", "invalidProof"],
    ["DOMAIN_REASON_EMAIL_MISMATCH", "emailMismatch"],
    ["DOMAIN_REASON_EMAIL_REGISTERED", "emailRegistered"],
    ["DOMAIN_REASON_WEAK_PASSWORD", "weakPassword"],
    ["DOMAIN_REASON_INVALID_CODE", "invalidCode"],
    ["DOMAIN_REASON_REGISTRATION_REQUIRED", "registrationRequired"],
    ["DOMAIN_REASON_ALREADY_MEMBER", "alreadyMember"],
    ["DOMAIN_REASON_ALREADY_PROCESSED", "alreadyProcessed"],
    ["DOMAIN_REASON_OWNER_TRANSFER_REQUIRED", "ownerTransferRequired"],
    ["DOMAIN_REASON_HANDLE_CONFLICT", "handleConflict"],
    ["DOMAIN_REASON_MAIL_UNAVAILABLE", "mailUnavailable"],
    ["DOMAIN_REASON_CAPACITY_EXHAUSTED", "capacityExhausted"],
  ];
  assert.deepEqual(
    known.map(([, key]) => key).sort(),
    Object.keys(en.Errors)
      .filter((key) => key !== "generic")
      .sort(),
  );
  for (const [reason, key] of known) {
    assert.equal(errorKey(reason), key);
    for (const messages of Object.values(translations)) {
      assert.ok(messages.Errors[key].trim().length > 0);
    }
  }
});

await test("missing, internal and unknown reasons use an honest generic message", () => {
  for (const reason of [
    null,
    "",
    "DOMAIN_REASON_INTERNAL",
    "DOMAIN_REASON_UNSPECIFIED",
    "DOMAIN_REASON_FUTURE_UNKNOWN",
    "INVALID_SESSION",
  ]) {
    assert.equal(errorKey(reason), "generic");
  }
  for (const messages of Object.values(translations)) {
    assert.ok(messages.Errors.generic.trim().length > 0);
  }
});

await test("message catalogs accept only exact supported locale identifiers", () => {
  assert.deepEqual([...SUPPORTED_LOCALES].sort(), [
    "en-US",
    "fr-FR",
    "ja-JP",
    "ko-KR",
  ]);
  for (const locale of SUPPORTED_LOCALES) {
    assert.equal(isSupportedLocale(locale), true);
  }
  for (const value of ["", "ko", "fr-CA", "de-DE", "KO-KR", "../ko-KR"]) {
    assert.equal(isSupportedLocale(value), false);
  }
});
