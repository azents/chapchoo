import assert from "node:assert/strict";
import test from "node:test";
import { inspect, format } from "node:util";
import { Metadata, status } from "@grpc/grpc-js";
import { initTRPC, TRPCError } from "@trpc/server";
import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import {
  writableRejection,
  privateResponse,
  attempt,
  reportTransportFailure,
} from "../dist/boundary.js";
import { webConfig } from "../dist/config.js";
import {
  cookiePolicy,
  readSessionCookie,
  sessionCookie,
  clearSessionCookie,
} from "../dist/cookies.js";
import {
  domainReason,
  RpcFailure,
  SafeRenderFailure,
  invalidSession,
  permissionDenied,
} from "../dist/errors.js";
import { call } from "../dist/rpc.js";
import { RequestSession, readOnlyIdentity } from "../dist/session.js";
import * as validate from "../dist/validation.js";

const A = "A".repeat(43);
const B = "B".repeat(43);
const C = "C".repeat(43);
const NOW = 1_000_000;
const user = {
  id: "user",
  email: "user@example.invalid",
  isAdmin: false,
  createdAt: { seconds: 1n, nanos: 0 },
};
function session(token = B, identity = user) {
  return {
    token,
    user: identity,
    createdAt: { seconds: 1n, nanos: 0 },
    expiresAt: { seconds: 99_999_999n, nanos: 0 },
  };
}
function config(
  app = "general",
  publicUrl = "http://localhost:3003",
  bindHost = "127.0.0.1",
) {
  return webConfig(app, {
    publicUrl,
    bindHost,
    backendUrl: "http://127.0.0.1:50051",
  });
}
const policy = cookiePolicy(config());

test("SSR failure retains its native source privately without leaking through default Error rendering", async () => {
  const source = new Error(
    "cookie-canary proof-canary sql-canary database-uri",
  );
  source.code = status.UNAVAILABLE;
  source.metadata = new Metadata();
  const original = console.error;
  const logs = [];
  console.error = (line) => {
    logs.push(line);
  };
  try {
    await assert.rejects(
      readOnlyIdentity(
        {
          identity: async () => {
            throw source;
          },
        },
        A,
        false,
      ),
      (failure) => {
        assert.ok(failure instanceof SafeRenderFailure);
        assert.ok(failure instanceof RpcFailure);
        assert.equal(failure.code, status.UNAVAILABLE);
        assert.equal(failure.reason, null);
        assert.equal(failure.sourceFailure().cause, source);
        const rendered = [
          inspect(failure, { depth: Infinity, showHidden: true }),
          format("%O", failure),
          format("%s", failure),
          JSON.stringify(failure),
          failure.stack,
          ...logs,
        ].join("\n");
        assert.doesNotMatch(
          rendered,
          /cookie-canary|proof-canary|sql-canary|database-uri/,
        );
        return true;
      },
    );
    assert.equal(logs.length, 1);
    assert.match(logs[0], /web_ssr_authentication_failure/);
  } finally {
    console.error = original;
  }
});

test("one handling boundary reports unknown transport failures without source secrets", () => {
  const original = console.error;
  const lines = [];
  console.error = (line) => {
    lines.push(line);
  };
  try {
    const native = new Error(
      "password=private-source database-uri bearer-credential",
    );
    const transport = new RpcFailure(status.UNAVAILABLE, null, native);
    reportTransportFailure(
      new TRPCError({ code: "SERVICE_UNAVAILABLE", cause: transport }),
    );
    reportTransportFailure(
      new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        cause: new RpcFailure(
          status.INTERNAL,
          "DOMAIN_REASON_INTERNAL",
          native,
        ),
      }),
    );
    reportTransportFailure(
      new TRPCError({ code: "BAD_REQUEST", cause: native }),
    );
    reportTransportFailure(
      new TRPCError({ code: "INTERNAL_SERVER_ERROR", cause: native }),
    );
    assert.equal(lines.length, 2);
    assert.match(lines[0], /web_rpc_transport_failure/);
    assert.match(lines[1], /web_request_processing_failure/);
    assert.doesNotMatch(
      lines.join(""),
      /private-source|database-uri|bearer-credential|password=/,
    );
    assert.equal(transport.cause, native);
  } finally {
    console.error = original;
  }
});
function requestSession(renew, token = A, admin = false) {
  return new RequestSession(
    { renew, identity: async () => user },
    token,
    policy,
    admin,
    () => NOW,
  );
}

test("local HTTP is loopback-only independently of production build mode", () => {
  assert.equal(config("general", "http://[::1]:3003", "::1").secure, false);
  for (const [url, bind] of [
    ["http://example.com", "127.0.0.1"],
    ["http://localhost", "0.0.0.0"],
    ["http://localhost", "192.168.1.1"],
    ["http://127.example.com", "127.0.0.1"],
  ]) {
    assert.throws(() => config("general", url, bind), /Invalid web/);
  }
  assert.equal(
    config("general", "https://example.com", "0.0.0.0").secure,
    true,
  );
});

test("cookie attributes, app isolation, actual expiry cap and matching deletion", () => {
  const admin = cookiePolicy(config("admin", "http://localhost:3004"));
  assert.notEqual(policy.name, admin.name);
  const local = sessionCookie(policy, A, NOW + 90 * 86_400_000, NOW);
  assert.match(local, /HttpOnly; SameSite=Lax; Max-Age=2592000/);
  assert.doesNotMatch(local, /Secure/);
  assert.match(sessionCookie(policy, A, NOW + 1250, NOW), /Max-Age=1;/);
  const generalSecure = cookiePolicy(config("general", "https://example.com"));
  assert.match(generalSecure.name, /^__Host-/);
  assert.equal(generalSecure.sameSite, "None");
  const adminSecure = cookiePolicy(
    config("admin", "https://example.com/admin"),
  );
  assert.equal(adminSecure.path, "/admin");
  assert.equal(adminSecure.sameSite, "Lax");
  assert.match(
    clearSessionCookie(adminSecure),
    /Path=\/admin; HttpOnly; SameSite=Lax; Secure; Max-Age=0/,
  );
  assert.equal(readSessionCookie(`${policy.name}=${A}`, policy), A);
  assert.equal(
    readSessionCookie(`${policy.name}=${A}; ${policy.name}=${B}`, policy),
    null,
  );
});

test("canonical metadata rejects absent, duplicate, unknown, numeric and unspecified reasons", () => {
  for (const value of [
    null,
    "DOMAIN_REASON_UNSPECIFIED",
    "DOMAIN_REASON_FUTURE",
    "1",
    "UNRECOGNIZED",
  ]) {
    const metadata = new Metadata();
    if (value !== null) {
      metadata.set("chapchoo-domain-reason", value);
    }
    assert.equal(domainReason(metadata), null);
  }
  const metadata = new Metadata();
  metadata.set("chapchoo-domain-reason", "DOMAIN_REASON_INVALID_SESSION");
  assert.equal(domainReason(metadata), "DOMAIN_REASON_INVALID_SESSION");
  metadata.add("chapchoo-domain-reason", "DOMAIN_REASON_INVALID_SESSION");
  assert.equal(domainReason(metadata), null);
});

test("protected operations renew exactly once, sharing only their request-local result", async () => {
  let renewals = 0;
  const context = requestSession(async (token) => {
    assert.equal(token, A);
    renewals += 1;
    return session();
  });
  const values = await Promise.all([
    context.authenticated(async (token) => token),
    context.authenticated(async (token) => token),
  ]);
  assert.deepEqual(values, [B, B]);
  assert.equal(renewals, 1);
  assert.match(context.outgoingCookie(), new RegExp(`=${B};`));
});

test("concurrent request contexts never share a caller credential", async () => {
  const barrier = Promise.withResolvers();
  const observed = [];
  const transport = async (token) => {
    observed.push(token);
    if (observed.length === 2) {
      barrier.resolve();
    }
    await barrier.promise;
    return session(token === A ? B : C);
  };
  const first = requestSession(transport, A);
  const second = requestSession(transport, C);
  assert.deepEqual(
    await Promise.all([
      first.authenticated(async (token) => token),
      second.authenticated(async (token) => token),
    ]),
    [B, C],
  );
  assert.deepEqual(observed, [A, C]);
});

test("committed renewal cookie survives ordinary business rejection", async () => {
  const context = requestSession(async () => session());
  await assert.rejects(
    context.authenticated(async () => {
      throw permissionDenied();
    }),
    { reason: "DOMAIN_REASON_PERMISSION_DENIED" },
  );
  assert.match(context.outgoingCookie(), new RegExp(`=${B};`));
});

test("post-renewal invalidation overrides the cookie candidate", async () => {
  const context = requestSession(async () => session());
  await assert.rejects(
    context.authenticated(async () => {
      throw invalidSession();
    }),
  );
  assert.match(context.outgoingCookie(), /Max-Age=0/);
  let businessCalls = 0;
  await assert.rejects(
    context.authenticated(async () => {
      businessCalls += 1;
    }),
  );
  assert.equal(businessCalls, 0);
  assert.match(context.outgoingCookie(), /Max-Age=0/);
});

test("auth rejection clears and stops dispatch; auth infrastructure failure retains cookie", async () => {
  for (const [error, clear] of [
    [invalidSession(), true],
    [
      new RpcFailure(
        status.UNAVAILABLE,
        null,
        new Error("private database source"),
      ),
      false,
    ],
  ]) {
    const context = requestSession(async () => {
      throw error;
    });
    let calls = 0;
    await assert.rejects(
      context.authenticated(async () => {
        calls += 1;
      }),
    );
    assert.equal(calls, 0);
    if (clear) {
      assert.match(context.outgoingCookie(), /Max-Age=0/);
    } else {
      assert.equal(context.outgoingCookie(), null);
    }
  }
});

test("SSR reads identity only, with no renewal or cookie disposition", async () => {
  let identities = 0;
  const transport = {
    identity: async (token) => {
      assert.equal(token, A);
      identities += 1;
      return user;
    },
    renew: () => {
      throw new Error("SSR must not renew");
    },
  };
  assert.equal((await readOnlyIdentity(transport, A, false)).id, user.id);
  assert.equal(identities, 1);
  await assert.rejects(readOnlyIdentity(transport, A, true), {
    reason: "DOMAIN_REASON_PERMISSION_DENIED",
  });
  assert.equal(await readOnlyIdentity(transport, null, true), null);
});

test("auth result publishes only identity; Admin cookie requires current admission", () => {
  const context = requestSession(async () => session());
  const result = context.acceptAuthentication(session());
  assert.deepEqual(result, user);
  assert.equal(Object.hasOwn(result, "token"), false);
  const admin = requestSession(async () => session(), A, true);
  assert.throws(() => admin.acceptAuthentication(session()), {
    reason: "DOMAIN_REASON_PERMISSION_DENIED",
  });
  assert.equal(admin.outgoingCookie(), null);
});

test("Node generated-response ingress rejects missing fields and unknown enums", () => {
  assert.throws(
    () => validate.session({ ...session(), user: undefined }),
    RpcFailure,
  );
  assert.throws(
    () =>
      validate.session({ ...session(), expiresAt: { seconds: 3n, nanos: -1 } }),
    RpcFailure,
  );
  assert.throws(
    () =>
      validate.team({
        id: "team",
        name: "name",
        handle: "handle",
        ownerUserId: "user",
        kind: 88,
        callerRole: 1,
      }),
    RpcFailure,
  );
  assert.throws(
    () => validate.member({ userId: "user", email: "email", role: 0 }),
    RpcFailure,
  );
  assert.throws(() => validate.preview({ usable: true }), RpcFailure);
  assert.throws(
    () =>
      validate.preview({
        usable: true,
        emailHint: "full-address@example.invalid",
        expiresAt: { seconds: 10n, nanos: 0 },
      }),
    RpcFailure,
  );
  assert.equal(
    validate.preview({
      usable: true,
      emailHint: "m***@example.invalid",
      expiresAt: { seconds: 10n, nanos: 0 },
    }).usable,
    true,
  );
  assert.throws(
    () => validate.preview({ usable: false, emailHint: "should not expose" }),
    RpcFailure,
  );
});

test("ordinary tRPC fetch headers retain renewal after typed business failure", async () => {
  let dispatches = 0;
  const t = initTRPC.context().create();
  const router = t.router({
    business: t.procedure.mutation(({ ctx }) =>
      attempt(() =>
        ctx.session.authenticated(async () => {
          dispatches += 1;
          throw permissionDenied();
        }),
      ),
    ),
  });
  async function handle(request) {
    const rejection = writableRejection(request, config());
    if (rejection) {
      return rejection;
    }
    const context = { session: requestSession(async () => session()) };
    const response = await fetchRequestHandler({
      req: request,
      endpoint: "/api/trpc",
      router,
      createContext: () => context,
      allowMethodOverride: true,
    });
    return privateResponse(response, context.session);
  }
  const base = "http://localhost:3003/api/trpc/business";
  assert.equal((await handle(new Request(base))).status, 405);
  assert.equal(
    (await handle(new Request(base, { method: "POST" }))).status,
    403,
  );
  assert.equal(
    (
      await handle(
        new Request(base, {
          method: "POST",
          headers: { Origin: "http://attacker.invalid" },
        }),
      )
    ).status,
    403,
  );
  assert.equal(dispatches, 0);
  const response = await handle(
    new Request(base, {
      method: "POST",
      headers: { Origin: config().origin, "Content-Type": "application/json" },
      body: "null",
    }),
  );
  assert.equal(dispatches, 1);
  assert.equal(response.status, 403);
  assert.match(response.headers.get("set-cookie"), new RegExp(`=${B};`));
  assert.equal(response.headers.get("cache-control"), "private, no-store");
  assert.doesNotMatch(await response.text(), new RegExp(B));
});

test("compound requests are rejected before identity creation and separate login retains its cookie", async () => {
  let contexts = 0;
  let loginCalls = 0;
  let businessCalls = 0;
  const t = initTRPC.context().create();
  const router = t.router({
    login: t.procedure.mutation(({ ctx }) => {
      loginCalls += 1;
      // The test router projects identity only; production uses SuperJSON for timestamps.
      return { id: ctx.session.acceptAuthentication(session()).id };
    }),
    business: t.procedure.mutation(({ ctx }) =>
      attempt(() =>
        ctx.session.authenticated(async () => {
          businessCalls += 1;
          return { completed: true };
        }),
      ),
    ),
  });
  async function handle(request) {
    const rejection = writableRejection(request, config());
    if (rejection) {
      return rejection;
    }
    contexts += 1;
    const context = { session: requestSession(async () => session(), null) };
    return privateResponse(
      await fetchRequestHandler({
        req: request,
        endpoint: "/api/trpc",
        router,
        createContext: () => context,
        allowMethodOverride: true,
        allowBatching: false,
      }),
      context.session,
    );
  }
  function request(path) {
    return new Request(`http://localhost:3003/api/trpc/${path}`, {
      method: "POST",
      headers: { Origin: config().origin, "Content-Type": "application/json" },
      body: "null",
    });
  }
  const combined = await handle(request("business,login?batch=1"));
  assert.equal(combined.status, 400);
  assert.equal(combined.headers.get("set-cookie"), null);
  assert.equal(contexts, 0);
  assert.equal(loginCalls, 0);
  assert.equal(businessCalls, 0);
  const rejected = await handle(request("business"));
  assert.equal(rejected.status, 401);
  assert.match(rejected.headers.get("set-cookie"), /Max-Age=0/);
  const authenticated = await handle(request("login"));
  assert.equal(authenticated.status, 200);
  assert.match(authenticated.headers.get("set-cookie"), new RegExp(`=${B};`));
  assert.equal(loginCalls, 1);
  assert.equal(businessCalls, 0);
  assert.doesNotMatch(await authenticated.text(), new RegExp(B));
});

test("call wrapper supplies fresh per-call metadata, finite deadlines, no credential defaults", async () => {
  const seen = [];
  function method(request, metadata, options, callback) {
    seen.push({ request, metadata, options });
    callback(null, { completed: true });
  }
  await Promise.all([
    call(method, {}, A),
    call(method, {}, B),
    call(method, {}),
  ]);
  assert.notEqual(seen[0].metadata, seen[1].metadata);
  assert.deepEqual(seen[0].metadata.get("authorization"), [`Bearer ${A}`]);
  assert.deepEqual(seen[1].metadata.get("authorization"), [`Bearer ${B}`]);
  assert.deepEqual(seen[2].metadata.get("authorization"), []);
  assert.equal(typeof seen[0].options.deadline, "number");
});

// This controlled cookie-jar/authority model exercises response disposition. It
// is not evidence that framework header tests reproduce real browser networking.
test("lost/reordered responses recover only inside the fixed previous-token overlap", async () => {
  let now = NOW;
  let current = A;
  let previous = null;
  let overlapEnd = 0;
  async function renew(token) {
    if (token === current) {
      if (current === A) {
        previous = A;
        current = B;
        overlapEnd = now + 300_000;
      }
      return session(current);
    }
    if (token === previous && now < overlapEnd) {
      return session(current);
    }
    throw invalidSession();
  }
  const make = (token) =>
    new RequestSession(
      { renew, identity: async () => user },
      token,
      policy,
      false,
      () => now,
    );
  // The server committed B but its response was lost; the browser retains A.
  await make(A).authenticated(async () => null);
  now += 1000;
  const within = make(A);
  await within.authenticated(async () => null);
  assert.equal(readSessionCookie(within.outgoingCookie(), policy), B);
  // An older successful A response can overwrite B in a browser cookie jar.
  let jar = A;
  assert.equal(await make(jar).authenticated(async (token) => token), B);
  now = overlapEnd;
  const late = make(jar);
  await assert.rejects(late.authenticated(async () => null));
  assert.match(late.outgoingCookie(), /Max-Age=0/);
  // A delayed invalid response may also clear a newer cookie; there is no
  // invented global monotonic cookie registry or unlimited recovery promise.
  jar = B;
  jar = readSessionCookie(late.outgoingCookie(), policy);
  assert.equal(jar, null);
});
