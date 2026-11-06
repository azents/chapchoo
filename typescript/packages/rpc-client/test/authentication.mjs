import { Buffer } from "node:buffer";

import { Kind, Operation } from "../dist/generated/authentication.js";

// Real generated-client calls against the owned Rust process. Device provider
// success is exercised by the injected Rust fixture, never a live OAuth request.
export async function verifyAuthentication({
  client,
  token,
  call,
  check,
  rejected,
  status,
}) {
  const scope = { personal: {} };
  await rejected(
    () => call(client, "list", { scope, limit: 0 }),
    status.UNAUTHENTICATED,
    "DOMAIN_REASON_INVALID_SESSION",
  );
  await rejected(
    () =>
      call(client, "startDevice", {
        scope,
        requestId: "anonymous-device",
        name: "Anonymous",
      }),
    status.UNAUTHENTICATED,
    "DOMAIN_REASON_INVALID_SESSION",
  );
  const key = Buffer.from("disposable-fixture-key");
  let registered;
  try {
    registered = await call(
      client,
      "registerKey",
      { scope, requestId: "node-register", name: "Unverified", key },
      token,
    );
    const replay = await call(
      client,
      "registerKey",
      { scope, requestId: "node-register", name: "Unverified", key },
      token,
    );
    check(
      replay.record?.authenticationId === registered.record?.authenticationId,
    );
  } finally {
    key.fill(0);
  }
  const id = registered.record?.authenticationId;
  check(typeof id === "string" && id.length > 0);
  const item = await call(
    client,
    "get",
    { scope, authenticationId: id },
    token,
  );
  check(item.kind === Kind.KIND_OPENAI_API_KEY && item.name === "Unverified");
  check(
    item.accountId === undefined &&
      item.email === undefined &&
      item.expiresAt === undefined,
  );
  check(!Object.hasOwn(item, "key") && !Object.hasOwn(item, "secret"));
  const listed = await call(client, "list", { scope, limit: 1 }, token);
  check(
    listed.items.length === 1 &&
      listed.items[0].id === id &&
      listed.nextAfter === id,
  );
  const end = await call(
    client,
    "list",
    { scope, limit: 1, after: listed.nextAfter },
    token,
  );
  check(end.items.length === 0 && end.nextAfter === undefined);
  const renamed = await call(
    client,
    "rename",
    {
      scope,
      requestId: "node-rename",
      authenticationId: id,
      expectedRequestId: "node-register",
      name: "Renamed",
    },
    token,
  );
  check(renamed.record?.authenticationId === id);
  await rejected(
    () =>
      call(
        client,
        "rename",
        {
          scope,
          requestId: "node-stale",
          authenticationId: id,
          expectedRequestId: "node-register",
          name: "Stale",
        },
        token,
      ),
    status.FAILED_PRECONDITION,
    "DOMAIN_REASON_INVALID_TRANSITION",
  );
  const replacement = Buffer.from("disposable-replacement-key");
  try {
    const replaced = await call(
      client,
      "replaceKey",
      {
        scope,
        requestId: "node-replace",
        authenticationId: id,
        expectedRequestId: "node-rename",
        key: replacement,
      },
      token,
    );
    check(replaced.record?.authenticationId === id);
  } finally {
    replacement.fill(0);
  }
  const deleted = await call(
    client,
    "delete",
    {
      scope,
      requestId: "node-delete",
      authenticationId: id,
      expectedRequestId: "node-replace",
    },
    token,
  );
  check(deleted.accepted?.deleted?.authenticationId === id);
  const recovered = await call(
    client,
    "requestResult",
    { requestId: "node-register" },
    token,
  );
  check(
    recovered.operation === Operation.OPERATION_REGISTER_KEY &&
      recovered.accepted?.record?.authenticationId === id &&
      recovered.acceptedAt !== undefined,
  );
  const absent = await call(
    client,
    "requestResult",
    { requestId: "never-accepted" },
    token,
  );
  check(absent.accepted === undefined);
  await rejected(
    () => call(client, "get", { scope, authenticationId: id }, token),
    status.NOT_FOUND,
    "DOMAIN_REASON_NOT_FOUND",
  );
}
