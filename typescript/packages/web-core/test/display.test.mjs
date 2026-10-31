import assert from "node:assert/strict";
import test from "node:test";
import {
  decodeDisplayCookie,
  detectDisplayValues,
  displayChoices,
  displayPart,
  displayPatchPayload,
  resolveGuestDisplay,
  sameDisplayValue,
} from "../dist/display.js";
import { display, user } from "../dist/validation.js";
import { invalidatesSession } from "../dist/auth-client.js";
import { displayMessages } from "../dist/display-messages.js";
const defaults = { language: "en", timeZone: "UTC", region: "en-US" };
test("browser language, regional format and time zone are detected independently", () => {
  assert.deepEqual(
    detectDisplayValues(["ko-KR", "en-US"], {
      locale: "fr-FR",
      timeZone: "America/New_York",
    }),
    { language: "ko", region: "fr-FR", timeZone: "America/New_York" },
  );
  assert.deepEqual(
    detectDisplayValues(["de-DE"], { locale: "de-DE", timeZone: "Not/AZone" }),
    {},
  );
  assert.deepEqual(detectDisplayValues(["ja-JP"], {}), { language: "ja" });
});
test("visitor explicit intent and projection resolve independently over server defaults", () => {
  assert.deepEqual(
    resolveGuestDisplay(
      defaults,
      { language: "ko" },
      { language: "fr", timeZone: "Asia/Seoul" },
    ),
    { language: "ko", timeZone: "Asia/Seoul", region: "en-US" },
  );
  assert.deepEqual(
    resolveGuestDisplay(
      defaults,
      { timeZone: "Europe/London", region: "fr-FR" },
      { timeZone: "Asia/Tokyo", language: "ja" },
    ),
    { language: "ja", timeZone: "Europe/London", region: "fr-FR" },
  );
  assert.deepEqual(resolveGuestDisplay(defaults, {}, {}), defaults);
});
test("cookie ingress validates each dimension and ignores unknown or malformed data", () => {
  assert.deepEqual(
    decodeDisplayCookie(
      encodeURIComponent(
        JSON.stringify({
          language: "ko",
          timeZone: "Not/AZone",
          region: "fr-FR",
          isAdmin: true,
        }),
      ),
    ),
    { language: "ko", region: "fr-FR" },
  );
  for (const value of [
    undefined,
    "not-json",
    "%broken",
    encodeURIComponent("null"),
    encodeURIComponent("[]"),
  ])
    assert.deepEqual(decodeDisplayCookie(value), {});
  for (const value of [
    null,
    [],
    true,
    { language: "unknown", timeZone: "+09:00", region: "en-GB" },
  ])
    assert.deepEqual(displayChoices(value), {});
  assert.equal(
    display({ language: "ko", timeZone: "Asia/Seoul", region: "en-US" }).region,
    "en-US",
  );
  assert.throws(() => display({ language: "ko" }));
});
test("IANA aliases compare as the same setting while independent dimensions remain different", () => {
  assert.equal(sameDisplayValue("timeZone", "UTC", "Etc/UTC"), true);
  assert.equal(
    sameDisplayValue("timeZone", "Asia/Seoul", "America/New_York"),
    false,
  );
  assert.equal(sameDisplayValue("region", "ko-KR", "en-US"), false);
  assert.equal(displayPart("timeZone", "Not/AZone"), false);
});
test("a stored zone absent from local Intl stays comparable without replacing it", (t) => {
  const Native = Intl.DateTimeFormat;
  t.mock.method(Intl, "DateTimeFormat", function (locale, options) {
    if (options?.timeZone === "Asia/Seoul")
      throw new RangeError("Simulated older timezone database");
    return new Native(locale, options);
  });
  assert.equal(sameDisplayValue("timeZone", "Asia/Seoul", "UTC"), false);
  assert.equal(sameDisplayValue("timeZone", "UTC", "Asia/Seoul"), false);
  assert.equal(sameDisplayValue("timeZone", "Asia/Seoul", "Asia/Seoul"), true);
});
test("partial transport fields preserve omission, and decoded settings are required", () => {
  assert.deepEqual(displayPatchPayload({ language: "fr" }), {
    language: "fr",
    timeZone: undefined,
    region: undefined,
  });
  assert.deepEqual(display(defaults), defaults);
  assert.throws(() => display(undefined));
  for (const timeZone of ["", "+09:00", "invalid zone", "A".repeat(129), null])
    assert.throws(() => display({ ...defaults, timeZone }));
});
test("backend response decoding preserves server-approved zones independently of local ICU", (t) => {
  const serverValue = { ...defaults, timeZone: "Asia/Seoul" };
  const Native = Intl.DateTimeFormat;
  t.mock.method(Intl, "DateTimeFormat", function (locale, options) {
    if (options?.timeZone === "Asia/Seoul")
      throw new RangeError("Simulated older Node timezone database");
    return new Native(locale, options);
  });
  assert.deepEqual(display(serverValue), serverValue);
  const identity = {
    id: "fixture-user",
    email: "fixture@example.invalid",
    isAdmin: false,
    createdAt: { seconds: 0n, nanos: 0 },
    display: serverValue,
  };
  assert.deepEqual(user(identity).display, serverValue);
  assert.equal(displayPart("timeZone", "Asia/Seoul"), false);
  assert.deepEqual(
    decodeDisplayCookie(encodeURIComponent(JSON.stringify(serverValue))),
    { language: "en", region: "en-US" },
  );
});
test("new protected mutations participate in invalid-session disposal without treating public methods as protected", () => {
  for (const app of ["general", "admin"])
    assert.equal(
      invalidatesSession(
        "DOMAIN_REASON_INVALID_SESSION",
        "auth.updateDisplay",
        app,
      ),
      true,
    );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_INVALID_SESSION",
      "displayDefaults.update",
      "admin",
    ),
    true,
  );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_INVALID_SESSION",
      "auth.methods",
      "general",
    ),
    false,
  );
});
test("Admin settings reads and current permission loss dispose only the Admin context", () => {
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_INVALID_SESSION",
      "displayDefaults.read",
      "admin",
    ),
    true,
  );
  for (const path of [
    "auth.updateDisplay",
    "displayDefaults.read",
    "displayDefaults.update",
  ])
    assert.equal(
      invalidatesSession("DOMAIN_REASON_PERMISSION_DENIED", path, "admin"),
      true,
    );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_PERMISSION_DENIED",
      "auth.updateDisplay",
      "general",
    ),
    false,
  );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_PERMISSION_DENIED",
      "teams.update",
      "general",
    ),
    false,
  );
});
test("all four display dictionaries have the same non-empty localized message keys", () => {
  const keys = Object.keys(displayMessages.en).sort();
  for (const [language, messages] of Object.entries(displayMessages)) {
    assert.deepEqual(Object.keys(messages).sort(), keys, language);
    for (const value of Object.values(messages))
      assert.ok(value.trim().length > 0);
  }
  assert.match(displayMessages.ko.proposal, /표시/);
});
