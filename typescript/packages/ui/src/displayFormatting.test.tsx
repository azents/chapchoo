import assert from "node:assert/strict";
import test from "node:test";
import { formatDisplayDate } from "./displayFormatting";

await test("date presentation preserves independent regional format and time zone", () => {
  const instant = Date.UTC(2026, 9, 31, 0);
  for (const region of ["en-US", "ko-KR", "ja-JP", "fr-FR"]) {
    assert.equal(
      formatDisplayDate(instant, region, "Asia/Seoul", "medium"),
      new Intl.DateTimeFormat(region, {
        dateStyle: "medium",
        timeStyle: "short",
        timeZone: "Asia/Seoul",
      }).format(instant),
    );
  }
});
await test("unavailable local timezone data reports no invented date or UTC substitution", (t) => {
  t.mock.method(Intl, "DateTimeFormat", function () {
    throw new RangeError("Simulated older timezone database");
  });
  assert.equal(formatDisplayDate(0, "en-US", "Asia/Seoul", "long"), null);
  assert.equal(formatDisplayDate(0, "en-US", "Asia/Seoul", "medium"), null);
});
