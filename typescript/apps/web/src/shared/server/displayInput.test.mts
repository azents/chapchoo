import assert from "node:assert/strict";
import test from "node:test";
import { displayInput, displayPatch } from "./displayInput.ts";

await test("display ingress accepts independent values and only valid renderable dimensions", () => {
  assert.deepEqual(
    displayInput.parse({
      language: "ko",
      timeZone: "America/New_York",
      region: "fr-FR",
    }),
    { language: "ko", timeZone: "America/New_York", region: "fr-FR" },
  );
  for (const patch of [
    { language: "xx" },
    { timeZone: "Not/AZone" },
    { region: "en-GB" },
    { language: null },
    { userId: "another-account" },
    {},
  ]) {
    assert.equal(displayPatch.safeParse(patch).success, false);
  }
  assert.deepEqual(displayPatch.parse({ region: "ja-JP" }), {
    region: "ja-JP",
  });
});
await test("account creation leaves absent dimensions for transaction-time defaults", () => {
  assert.deepEqual(displayInput.partial().parse({ language: "ko" }), {
    language: "ko",
  });
  assert.deepEqual(displayInput.partial().parse({}), {});
});
