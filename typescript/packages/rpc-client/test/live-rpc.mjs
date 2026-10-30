import process from "node:process";

import { credentials, Metadata, status } from "@grpc/grpc-js";

import {
  AdminClient,
  AuthClient,
  DomainReason,
  GeneralClient,
  InvitationAction,
  TeamKind,
  TeamRole,
} from "../dist/generated/accounts.js";

const reasonKey = "chapchoo-domain-reason";
const clients = [];
let checks = 0;

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
  check(generalEndpoint !== adminEndpoint);
  const authGeneral = client(AuthClient, generalEndpoint);
  const authAdmin = client(AuthClient, adminEndpoint);
  const general = client(GeneralClient, generalEndpoint);
  const admin = client(AdminClient, adminEndpoint);
  const wrongAdmin = client(AdminClient, generalEndpoint);
  const wrongGeneral = client(GeneralClient, adminEndpoint);

  for (const auth of [authGeneral, authAdmin]) {
    const methods = await call(auth, "methods", {});
    check(methods.password === true);
    check(methods.emailCode === false);
    check(methods.signupEmail === false);
  }
  const setup = await call(admin, "setupStatus", {});
  check(setup.available === true);
  const adminEmail = "live-administrator@example.com";
  const userEmail = "live-member@example.com";
  const password = "Disposable-Fixture1!";
  const administrator = await call(admin, "initialize", {
    installationProof: setupProof,
    email: adminEmail,
    password,
  });
  check(
    typeof administrator.token === "string" && administrator.token.length > 0,
  );
  check(administrator.user?.isAdmin === true);
  const identity = await call(authGeneral, "identity", {}, administrator.token);
  check(identity.id === administrator.user.id && identity.isAdmin === true);

  const issued = await call(
    admin,
    "issueSignupGrant",
    { email: userEmail },
    administrator.token,
  );
  check(typeof issued.proof === "string" && issued.proof.length > 0);
  const preview = await call(general, "previewSignup", { proof: issued.proof });
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
  const member = await call(general, "redeemSignup", {
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
        admin,
        "issueSignupGrant",
        { email: "live-forbidden@example.com" },
        member.token,
      ),
    status.PERMISSION_DENIED,
    "DOMAIN_REASON_PERMISSION_DENIED",
  );

  const team = await call(
    general,
    "createTeam",
    { name: "Live RPC Team", handle: "live-rpc-team" },
    administrator.token,
  );
  check(team.kind === TeamKind.TEAM_KIND_COLLABORATIVE);
  check(team.callerRole === TeamRole.TEAM_ROLE_OWNER);
  const teams = await call(general, "listTeams", {}, administrator.token);
  check(teams.items.some((item) => item.id === team.id));
  const updated = await call(
    general,
    "updateTeam",
    { teamId: team.id, name: "Updated Live RPC Team" },
    administrator.token,
  );
  check(updated.name === "Updated Live RPC Team");
  check(updated.handle === team.handle);
  await rejected(
    () =>
      call(
        general,
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
        general,
        "setMemberRole",
        { teamId: team.id, userId: member.user.id, role: 12345 },
        administrator.token,
      ),
    status.INVALID_ARGUMENT,
    "DOMAIN_REASON_INVALID_INPUT",
  );

  const invitation = await call(
    general,
    "inviteMember",
    { teamId: team.id, email: userEmail },
    administrator.token,
  );
  const received = await call(general, "receivedInvitations", {}, member.token);
  check(received.items.some((item) => item.id === invitation.id));
  await call(
    general,
    "processInvitation",
    {
      invitationId: invitation.id,
      action: InvitationAction.INVITATION_ACTION_ACCEPT,
    },
    member.token,
  );
  const joined = await call(general, "listTeams", {}, member.token);
  check(
    joined.items.some(
      (item) =>
        item.id === team.id && item.callerRole === TeamRole.TEAM_ROLE_MEMBER,
    ),
  );
  await rejected(
    () => call(general, "leaveTeam", { teamId: team.id }, administrator.token),
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
    () => call(admin, "listSignupGrants", {}, passwordSession.token),
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
  await rejected(
    () => call(wrongAdmin, "setupStatus", {}),
    status.UNIMPLEMENTED,
    null,
  );
  await rejected(
    () => call(wrongGeneral, "listTeams", {}, administrator.token),
    status.UNIMPLEMENTED,
    null,
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
  process.stderr.write("Live RPC verification failed.\n");
  process.exitCode = 1;
} finally {
  for (const instance of clients) {
    instance.close();
  }
}
