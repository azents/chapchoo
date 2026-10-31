import assert from "node:assert/strict";
import test from "node:test";
import {
  initialQuery,
  inventoryOffset,
  loginMethod,
  readSelection,
  selectionUrl,
  serverSearchValues,
} from "../dist/page-state.js";

test("inventory URL parsing accepts only bounded canonical offsets", () => {
  for (const value of [
    null,
    "",
    "-1",
    "01",
    " 25",
    "25x",
    "9223372036854775808",
  ])
    assert.equal(inventoryOffset.parse(value), 0n);
  assert.equal(inventoryOffset.parse("25"), 25n);
  assert.equal(
    inventoryOffset.parse("9223372036854775807"),
    9223372036854775807n,
  );
});
test("selection URLs preserve mount, unrelated selections and fragment without serializing default state", () => {
  assert.equal(
    selectionUrl(
      "https://example.test/admin/password-resets?other=kept#list",
      "offset",
      25n,
      inventoryOffset,
    ),
    "/admin/password-resets?other=kept&offset=25#list",
  );
  assert.equal(
    selectionUrl("/admin?offset=25&other=kept", "offset", 0n, inventoryOffset),
    "/admin?other=kept",
  );
  assert.equal(
    readSelection(new URLSearchParams("method=code"), "method", loginMethod),
    "code",
  );
  assert.equal(
    readSelection(new URLSearchParams("method=unknown"), "method", loginMethod),
    "password",
  );
});
test("query handoff uses the exact initial object and timestamp without mount/reconnect reads", () => {
  const data = { items: [] };
  const options = initialQuery(data, 123);
  assert.equal(options.initialData, data);
  assert.equal(options.initialDataUpdatedAt, 123);
  assert.equal(options.refetchOnMount, false);
  assert.equal(options.refetchOnReconnect, false);
});
test("server and browser select the same first duplicate URL value", () => {
  const server = serverSearchValues({
    offset: ["25", "50"],
    method: ["code", "password"],
  });
  const browser = new URLSearchParams(
    "offset=25&offset=50&method=code&method=password",
  );
  assert.equal(
    readSelection(server, "offset", inventoryOffset),
    readSelection(browser, "offset", inventoryOffset),
  );
  assert.equal(
    readSelection(server, "method", loginMethod),
    readSelection(browser, "method", loginMethod),
  );
});
