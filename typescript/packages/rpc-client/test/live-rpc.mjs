import process from "node:process";

import { credentials, Metadata, status } from "@grpc/grpc-js";
import { AgentsClient } from "../dist/generated/computers.js";

import {
  AuthenticationClient,
  PersonalPreferencesClient,
  AccountLifecycleClient,
  TeamManagementClient,
  InvitationManagementClient,
  InstallationClient,
  AccountGrantsClient,
  DisplayDefaultsClient,
  DomainReason,
  InvitationAction,
  TeamKind,
  TeamRole,
} from "../dist/generated/accounts.js";

const reasonKey = "chapchoo-domain-reason";
const clients = [];
let checks = 0;
let lastMethod = "not-started";

function check(condition) {
  // Never attach an RPC response, credential, or dependency error to failures.
  if (!condition) {
    throw new Error("Live RPC assertion failed");
  }
  checks += 1;
}

function required(name) {
  const value = process.env[name];
  check(typeof value === "string" && value.length > 0);
  return value;
}

function localEndpoint(name) {
  const value = required(name);
  check(/^(?:127\.0\.0\.1|localhost|\[::1\]):[0-9]+$/.test(value));
  return value;
}

function client(Type, endpoint) {
  const instance = new Type(endpoint, credentials.createInsecure());
  clients.push(instance);
  return instance;
}

function metadata(token) {
  const value = new Metadata();
  if (token !== undefined) {
    value.set("authorization", `Bearer ${token}`);
  }
  return value;
}

function call(instance, method, request, token) {
  lastMethod = method;
  // A new metadata object and a finite deadline belong to every individual call.
  // No client default metadata or mutable current-user context is installed.
  return new Promise((resolve, reject) => {
    instance[method](
      request,
      metadata(token),
      { deadline: new Date(Date.now() + 15_000) },
      (error, response) => {
        if (error !== null) {
          reject(error);
        } else {
          resolve(response);
        }
      },
    );
  });
}

function domainReason(error) {
  const values = error?.metadata?.get(reasonKey);
  if (!Array.isArray(values) || values.length !== 1) {
    return null;
  }
  const value = values[0];
  if (
    typeof value !== "string" ||
    !value.startsWith("DOMAIN_REASON_") ||
    !Object.hasOwn(DomainReason, value) ||
    typeof DomainReason[value] !== "number" ||
    DomainReason[value] <= 0
  ) {
    return null;
  }
  return value;
}

async function rejected(operation, code, reason) {
  let failure;
  try {
    await operation();
  } catch (error) {
    failure = error;
  }
  check(failure !== undefined);
  check(failure.code === code);
  check(domainReason(failure) === reason);
}

async function main() {
  const generalEndpoint = localEndpoint("CHAPCHOO_TEST_GENERAL_ENDPOINT");
  const adminEndpoint = localEndpoint("CHAPCHOO_TEST_ADMIN_ENDPOINT");
  const setupProof = required("CHAPCHOO_TEST_SETUP_PROOF");
  const authGeneral = client(AuthenticationClient, generalEndpoint);
  const authAdmin = client(AuthenticationClient, adminEndpoint);
  const personalPreferences = client(
    PersonalPreferencesClient,
    generalEndpoint,
  );
  const accountLifecycle = client(AccountLifecycleClient, generalEndpoint);
  const teams = client(TeamManagementClient, generalEndpoint);
  const invitations = client(InvitationManagementClient, generalEndpoint);
  const installation = client(InstallationClient, adminEndpoint);
  const accountGrants = client(AccountGrantsClient, adminEndpoint);
  const displayDefaults = client(DisplayDefaultsClient, adminEndpoint);
  const agents = client(AgentsClient, generalEndpoint);

  for (const auth of [authGeneral, authAdmin]) {
    const methods = await call(auth, "methods", {});
    check(methods.password === true);
    check(methods.emailCode === false);
    check(methods.signupEmail === false);
  }
  const setup = await call(installation, "setupStatus", {});
  check(setup.available === true);
  const adminEmail = "live-administrator@example.com";
  const userEmail = "live-member@example.com";
  const password = "Disposable-Fixture1!";
  const administrator = await call(installation, "initialize", {
    installationProof: setupProof,
    email: adminEmail,
    password,
  });
  check(
    typeof administrator.token === "string" && administrator.token.length > 0,
  );
  check(administrator.user?.isAdmin === true);
  const initialDefaults = await call(
    personalPreferences,
    "displayDefaults",
    {},
  );
  const updatedDefaults = await call(
    displayDefaults,
    "updateDisplayDefaults",
    { language: "ko" },
    administrator.token,
  );
  check(updatedDefaults.language === "ko");
  check(updatedDefaults.timeZone === initialDefaults.timeZone);
  const preferences = await call(
    personalPreferences,
    "updateDisplay",
    { language: "fr" },
    administrator.token,
  );
  check(preferences.display?.language === "fr");
  const identity = await call(authGeneral, "identity", {}, administrator.token);
  check(identity.id === administrator.user.id && identity.isAdmin === true);
  await rejected(
    () => call(agents, "listProfileImages", { limit: 2 }),
    status.UNAUTHENTICATED,
    "DOMAIN_REASON_INVALID_SESSION",
  );
  const catalogue = await call(
    agents,
    "listProfileImages",
    { limit: 20 },
    administrator.token,
  );
  check(catalogue.items.length === 6 && catalogue.nextAfter === undefined);
  check(new Set(catalogue.items.map((image) => image.id)).size === 6);
  check(
    catalogue.items.every(
      (image) =>
        image.id.length > 0 &&
        image.internalName.length > 0 &&
        image.assetKey.length > 0,
    ),
  );
  const imagePage = await call(
    agents,
    "listProfileImages",
    { limit: 2 },
    administrator.token,
  );
  check(
    imagePage.items.length === 2 &&
      imagePage.nextAfter === imagePage.items[1].id,
  );
  const nextImagePage = await call(
    agents,
    "listProfileImages",
    {
      limit: 2,
      after: imagePage.nextAfter,
    },
    administrator.token,
  );
  check(
    nextImagePage.items.length === 2 &&
      nextImagePage.items[0].id === catalogue.items[2].id,
  );
  const endImages = await call(
    agents,
    "listProfileImages",
    {
      limit: 2,
      after: catalogue.items[5].id,
    },
    administrator.token,
  );
  check(endImages.items.length === 0 && endImages.nextAfter === undefined);
  await rejected(
    () => call(agents, "listProfileImages", { limit: 21 }, administrator.token),
    status.INVALID_ARGUMENT,
    null,
  );
  await rejected(
    () =>
      call(
        agents,
        "listProfileImages",
        { limit: 2, after: identity.id },
        administrator.token,
      ),
    status.INVALID_ARGUMENT,
    "DOMAIN_REASON_INVALID_INPUT",
  );

  const issued = await call(
    accountGrants,
    "issueSignupGrant",
    { email: userEmail },
    administrator.token,
  );
  check(typeof issued.proof === "string" && issued.proof.length > 0);
  const preview = await call(accountLifecycle, "previewSignup", {
    proof: issued.proof,
  });
  check(preview.usable === true);
  check(
    typeof preview.emailHint === "string" &&
      preview.emailHint !== userEmail &&
      preview.emailHint.includes("***"),
  );
  check(
    typeof preview.expiresAt?.seconds === "bigint" &&
      preview.expiresAt.seconds > 0n &&
      Number.isInteger(preview.expiresAt.nanos),
  );
  const member = await call(accountLifecycle, "redeemSignup", {
    proof: issued.proof,
    email: userEmail,
    password,
  });
  check(typeof member.token === "string" && member.token.length > 0);
  check(member.user?.isAdmin === false);
  check(member.user?.id !== administrator.user.id);
  await rejected(
    () =>
      call(
        accountGrants,
        "issueSignupGrant",
        { email: "live-forbidden@example.com" },
        member.token,
      ),
    status.PERMISSION_DENIED,
    "DOMAIN_REASON_PERMISSION_DENIED",
  );

  const team = await call(
    teams,
    "createTeam",
    { name: "Live RPC Team", handle: "live-rpc-team" },
    administrator.token,
  );
  check(team.kind === TeamKind.TEAM_KIND_COLLABORATIVE);
  check(team.callerRole === TeamRole.TEAM_ROLE_OWNER);
  check(
    (
      await call(
        agents,
        "checkHandle",
        {
          teamId: team.id,
          candidate: "live-agent",
        },
        administrator.token,
      )
    ).available === true,
  );
  await rejected(
    () =>
      call(
        agents,
        "checkHandle",
        { teamId: team.id, candidate: "live-agent" },
        member.token,
      ),
    status.NOT_FOUND,
    "DOMAIN_REASON_NOT_FOUND",
  );
  await rejected(
    () =>
      call(
        agents,
        "checkHandle",
        { teamId: team.id, candidate: " Invalid" },
        administrator.token,
      ),
    status.INVALID_ARGUMENT,
    "DOMAIN_REASON_INVALID_INPUT",
  );
  const teamList = await call(teams, "listTeams", {}, administrator.token);
  check(teamList.items.some((item) => item.id === team.id));
  const updated = await call(
    teams,
    "updateTeam",
    { teamId: team.id, name: "Updated Live RPC Team" },
    administrator.token,
  );
  check(updated.name === "Updated Live RPC Team");
  check(updated.handle === team.handle);
  await rejected(
    () =>
      call(
        teams,
        "updateTeam",
        { teamId: team.id, name: "" },
        administrator.token,
      ),
    status.INVALID_ARGUMENT,
    "DOMAIN_REASON_INVALID_INPUT",
  );
  await rejected(
    () =>
      call(
        teams,
        "setMemberRole",
        { teamId: team.id, userId: member.user.id, role: 12345 },
        administrator.token,
      ),
    status.INVALID_ARGUMENT,
    "DOMAIN_REASON_INVALID_INPUT",
  );

  const invitation = await call(
    invitations,
    "inviteMember",
    { teamId: team.id, email: userEmail },
    administrator.token,
  );
  const received = await call(
    invitations,
    "receivedInvitations",
    {},
    member.token,
  );
  check(received.items.some((item) => item.id === invitation.id));
  await call(
    invitations,
    "processInvitation",
    {
      invitationId: invitation.id,
      action: InvitationAction.INVITATION_ACTION_ACCEPT,
    },
    member.token,
  );
  const joined = await call(teams, "listTeams", {}, member.token);
  check(
    joined.items.some(
      (item) =>
        item.id === team.id && item.callerRole === TeamRole.TEAM_ROLE_MEMBER,
    ),
  );
  await rejected(
    () => call(teams, "leaveTeam", { teamId: team.id }, administrator.token),
    status.FAILED_PRECONDITION,
    "DOMAIN_REASON_OWNER_TRANSFER_REQUIRED",
  );

  const passwordSession = await call(authAdmin, "passwordLogin", {
    email: userEmail,
    password,
  });
  check(passwordSession.user?.id === member.user.id);
  check(passwordSession.user?.isAdmin === false);
  for (const auth of [authGeneral, authAdmin]) {
    const current = await call(auth, "identity", {}, passwordSession.token);
    check(current.id === member.user.id && current.isAdmin === false);
  }
  await rejected(
    () => call(accountGrants, "listSignupGrants", {}, passwordSession.token),
    status.PERMISSION_DENIED,
    "DOMAIN_REASON_PERMISSION_DENIED",
  );
  await rejected(
    () => call(authAdmin, "sendCode", { email: userEmail }),
    status.FAILED_PRECONDITION,
    "DOMAIN_REASON_MAIL_UNAVAILABLE",
  );
  await call(authAdmin, "logout", {}, passwordSession.token);
  for (const auth of [authGeneral, authAdmin]) {
    await rejected(
      () => call(auth, "identity", {}, passwordSession.token),
      status.UNAUTHENTICATED,
      "DOMAIN_REASON_INVALID_SESSION",
    );
  }
  // Logging out one session must not revoke a separate signup-created session.
  const stillSignedIn = await call(authGeneral, "identity", {}, member.token);
  check(stillSignedIn.id === member.user.id);
  // Alternating callers on one shared client never changes another call's user.
  const stillAdmin = await call(authAdmin, "identity", {}, administrator.token);
  check(stillAdmin.id === administrator.user.id && stillAdmin.isAdmin === true);
  await rejected(
    () => call(authGeneral, "identity", {}),
    status.UNAUTHENTICATED,
    "DOMAIN_REASON_INVALID_SESSION",
  );
  // Unknown or missing trailing metadata stays an unclassified transport error.
  // This harness does not invent product error semantics from arbitrary strings.
  const unknown = new Metadata();
  unknown.set(reasonKey, "DOMAIN_REASON_FUTURE_UNRECOGNIZED");
  check(domainReason({ metadata: unknown }) === null);
  check(domainReason({ metadata: new Metadata() }) === null);
  process.stdout.write(`Live RPC verification passed (${checks} checks).\n`);
}

try {
  await main();
} catch {
  // A raw gRPC error can retain request metadata. Never print it or its cause.
  process.stderr.write(
    `Live RPC verification failed after ${checks} checks (${lastMethod}).\n`,
  );
  process.exitCode = 1;
} finally {
  for (const instance of clients) {
    instance.close();
  }
}
