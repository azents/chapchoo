import assert from "node:assert/strict";
import test from "node:test";
import { preferredTeam, workspacePreference } from "../dist/workspace.js";
const personal = { id: "personal", kind: 1 };
const shared = { id: "shared", kind: 2 };
const teams = [personal, shared];
const choice = (user, team) =>
  encodeURIComponent(JSON.stringify({ user, team }));
test("remembered workspace is only a preference inside the current identity and accessible Team list", () => {
  assert.equal(
    preferredTeam(choice("owner", "shared"), "owner", teams),
    shared,
  );
  for (const value of [
    undefined,
    "invalid",
    "%bad",
    "null",
    choice("other", "shared"),
    choice("owner", "removed"),
    choice("owner", 123),
  ]) {
    assert.equal(preferredTeam(value, "owner", teams), personal);
  }
  assert.equal(preferredTeam(choice("owner", "shared"), "owner", []), null);
});
test("workspace cookie carries only a bounded identity/context preference at the host root", () => {
  const cookie = workspacePreference("owner", "shared", true);
  assert.match(cookie, /Path=\/; SameSite=Lax; Max-Age=2592000; Secure$/);
  const value = cookie.split(";")[0].split("=")[1];
  assert.equal(preferredTeam(value, "owner", teams), shared);
  assert.doesNotMatch(workspacePreference("owner", "shared", false), /Secure/);
});
