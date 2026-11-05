import assert from "node:assert/strict";
import test from "node:test";
import {
  preferredTeam,
  workspacePreference,
  teamPath,
  routeTeamHandle,
  agentsPath,
  newAgentPath,
  agentPath,
  agentChatPath,
  profileRoute,
  profileNavigation,
} from "../dist/workspace.js";
import { shellCopy } from "../dist/shell-messages.js";
test("Profile has independent account/display/invitation routes, not a personal Team selection", () => {
  const menu = profileNavigation(
    "/app",
    "/profile/preferences",
    shellCopy("en"),
    true,
  );
  assert.deepEqual(
    menu.map(({ key, href, active }) => ({ key, href, active })),
    [
      { key: "account", href: "/app/profile", active: false },
      { key: "display", href: "/app/profile/preferences", active: true },
      { key: "invitations", href: "/app/profile/invitations", active: false },
    ],
  );
  assert.deepEqual(
    profileNavigation("/admin", "/profile", shellCopy("en"), false).map(
      (item) => item.key,
    ),
    ["account", "display"],
  );
  for (const route of [
    "/profile",
    "/profile/preferences",
    "/profile/invitations",
  ])
    assert.equal(profileRoute(route), true);
  for (const route of ["/profiles", "/teams/my/agents", "/"])
    assert.equal(profileRoute(route), false);
});
const personal = { id: "personal", kind: 1 };
const shared = { id: "shared", kind: 2 };
const teams = [personal, shared];
test("navigation decodes the same valid Team path selector as server route params", () => {
  assert.equal(routeTeamHandle("/teams/%72esearch/agents"), "research");
  assert.equal(routeTeamHandle("/teams/my/new-agent"), "my");
  for (const path of [
    "/agents?team=id",
    "/teams/%ZZ/agents",
    "/teams/%2F/agents",
    "/teams/Invalid/agents",
  ])
    assert.equal(routeTeamHandle(path), null);
});
test("canonical Team and Agent routes are mount-aware and keep IDs out of paths", () => {
  const collaborative = { ...shared, handle: "engineering" };
  assert.equal(teamPath("", personal), "/teams/my");
  assert.equal(
    agentsPath("/general", collaborative),
    "/general/teams/engineering/agents",
  );
  assert.equal(
    newAgentPath("https://example.invalid/general/", collaborative),
    "https://example.invalid/general/teams/engineering/new-agent",
  );
  assert.equal(
    agentPath("/general", collaborative, "new"),
    "/general/teams/engineering/agents/new",
  );
  assert.equal(
    agentChatPath("/general", collaborative, "research"),
    "/general/teams/engineering/agents/research/chat",
  );
  for (const handle of ["id:uuid", "Invalid", "x", "../private"]) {
    assert.throws(() => teamPath("", handle));
  }
  assert.throws(() => agentPath("", "my", "Invalid"));
});
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
