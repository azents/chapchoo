import assert from "node:assert/strict";
import test from "node:test";
import { isSupportedLocale } from "./locale.ts";

await test("recognizes only supported locales", () => {
  assert.equal(isSupportedLocale("ko-KR"), true);
  assert.equal(isSupportedLocale("ko"), false);
  assert.equal(isSupportedLocale("de-DE"), false);
});
