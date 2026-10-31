import assert from "node:assert/strict";
import test from "node:test";
import {
  createAuthenticationClient,
  invalidatesSession,
} from "../dist/auth-client.js";
import { TRPCClientError } from "@trpc/client";

const identityKey = [["auth", "identity"], { type: "query" }];
function failure(path, reason) {
  return TRPCClientError.from({
    error: {
      message: "Request failed",
      code: -32001,
      data: { path, reason, code: "UNAUTHORIZED" },
    },
  });
}
function fixture(authenticated = true) {
  const locations = [];
  return {
    ...createAuthenticationClient(
      identityKey,
      authenticated,
      "/app",
      (path) => locations.push(path),
      "general",
    ),
    locations,
  };
}

void test("only protected invalid-session reasons end authentication", () => {
  for (const path of ["auth.identity", "teams.list", "invitations.process"]) {
    assert.equal(
      invalidatesSession("DOMAIN_REASON_INVALID_SESSION", path, "general"),
      true,
    );
  }
  for (const reason of [
    "DOMAIN_REASON_INVALID_CREDENTIALS",
    "DOMAIN_REASON_INVALID_CODE",
    "DOMAIN_REASON_PERMISSION_DENIED",
    null,
  ]) {
    assert.equal(invalidatesSession(reason, "teams.list", "general"), false);
  }
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_INVALID_SESSION",
      "auth.passwordLogin",
      "general",
    ),
    false,
  );
  assert.equal(
    invalidatesSession("DOMAIN_REASON_INVALID_SESSION", null, "general"),
    false,
  );
});

void test("query and mutation session failures share cleanup and one navigation", async () => {
  const { queryClient: cache, locations } = fixture();
  cache.setQueryData(identityKey, { id: "user" });
  cache.setQueryData(["private-team"], { name: "Private" });
  const failedQuery = cache.fetchQuery({
    queryKey: ["business"],
    queryFn: () =>
      Promise.reject(failure("teams.list", "DOMAIN_REASON_INVALID_SESSION")),
  });
  await assert.rejects(failedQuery);
  const mutation = cache.getMutationCache().build(cache, {
    mutationFn: () =>
      Promise.reject(
        failure("invitations.process", "DOMAIN_REASON_INVALID_SESSION"),
      ),
  });
  await assert.rejects(mutation.execute(void 0));
  assert.deepEqual(locations, ["/app/login"]);
  assert.equal(cache.getQueryData(identityKey), null);
  assert.equal(cache.getQueryData(["private-team"]), void 0);
  cache.clear();
});

void test("invalid credentials and transport failures retain identity without replay", async () => {
  const { queryClient: cache, locations } = fixture();
  const user = { id: "user" };
  cache.setQueryData(identityKey, user);
  let calls = 0;
  for (const [path, reason] of [
    ["auth.passwordLogin", "DOMAIN_REASON_INVALID_CREDENTIALS"],
    ["teams.create", "DOMAIN_REASON_PERMISSION_DENIED"],
  ]) {
    const mutation = cache.getMutationCache().build(cache, {
      mutationFn: () => {
        calls += 1;
        return Promise.reject(failure(path, reason));
      },
    });
    await assert.rejects(mutation.execute(void 0));
  }
  await assert.rejects(
    cache.fetchQuery({
      queryKey: ["temporary-error"],
      queryFn: () => {
        calls += 1;
        return Promise.reject(new Error("Network unavailable"));
      },
    }),
  );
  assert.equal(calls, 3);
  assert.deepEqual(locations, []);
  assert.equal(cache.getQueryData(identityKey), user);
  cache.clear();
});

void test("explicit identity recheck uses one query and handles signed-out success", async () => {
  const { queryClient: cache, locations } = fixture();
  cache.setQueryData(identityKey, { id: "user" });
  let calls = 0;
  await cache.fetchQuery({
    queryKey: identityKey,
    queryFn: () => {
      calls += 1;
      return Promise.resolve(null);
    },
  });
  assert.equal(calls, 1);
  assert.deepEqual(locations, ["/app/login"]);
  assert.equal(cache.getQueryData(identityKey), null);
  cache.clear();
  const signedOut = fixture(false);
  await signedOut.queryClient.fetchQuery({
    queryKey: identityKey,
    queryFn: () => Promise.resolve(null),
  });
  assert.deepEqual(signedOut.locations, []);
  signedOut.queryClient.clear();
});

void test("authentication completion drops previous account data and returns to root", () => {
  const { queryClient: cache, navigation, locations } = fixture();
  cache.setQueryData(identityKey, { id: "old-user" });
  cache.setQueryData(["old-data"], "private");
  navigation.authenticated();
  navigation.authenticated();
  assert.deepEqual(locations, ["/app/"]);
  assert.equal(cache.getQueryData(["old-data"]), void 0);
  cache.clear();
});

void test("logout immediately discards cached identity and submits only one local POST", () => {
  const locations = [];
  const posts = [];
  const { queryClient: cache, navigation } = createAuthenticationClient(
    identityKey,
    true,
    "/app",
    (path) => locations.push(path),
    "general",
    (path) => posts.push(path),
  );
  cache.setQueryData(identityKey, { id: "user" });
  navigation.logout();
  navigation.logout();
  assert.equal(cache.getQueryData(identityKey), null);
  assert.deepEqual(posts, ["/app/api/logout"]);
  assert.deepEqual(locations, []);
  cache.clear();
});

void test("reset clears user and business caches without navigation or stale SSR revival", async () => {
  const { queryClient: cache, navigation, locations } = fixture();
  cache.setQueryData(identityKey, { id: "old-user" });
  cache.setQueryData(["proof"], { usable: true });
  navigation.clearIdentity();
  assert.equal(cache.getQueryData(identityKey), null);
  assert.equal(cache.getQueryData(["proof"]), void 0);
  await cache.fetchQuery({
    queryKey: identityKey,
    queryFn: () => Promise.resolve(null),
  });
  assert.deepEqual(locations, []);
  cache.clear();
});

void test("a recovered identity becomes the current lifecycle before subsequent session loss", async () => {
  for (const app of ["general", "admin"]) {
    const locations = [];
    const { queryClient: cache } = createAuthenticationClient(
      identityKey,
      false,
      "/mount",
      (path) => locations.push(path),
      app,
    );
    await cache.fetchQuery({
      queryKey: identityKey,
      queryFn: () => Promise.resolve({ id: "recovered-user" }),
    });
    assert.deepEqual(locations, []);
    await cache.fetchQuery({
      queryKey: identityKey,
      queryFn: () => Promise.resolve(null),
    });
    assert.deepEqual(locations, ["/mount/login"]);
    cache.clear();
  }
});

void test("Admin policy invalidates only its own protected session or lost administrator identity", async () => {
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_INVALID_SESSION",
      "signupGrants.list",
      "admin",
    ),
    true,
  );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_INVALID_SESSION",
      "resetGrants.revoke",
      "admin",
    ),
    true,
  );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_PERMISSION_DENIED",
      "auth.identity",
      "admin",
    ),
    true,
  );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_PERMISSION_DENIED",
      "signupGrants.issue",
      "admin",
    ),
    false,
  );
  assert.equal(
    invalidatesSession(
      "DOMAIN_REASON_INVALID_CREDENTIALS",
      "auth.passwordLogin",
      "admin",
    ),
    false,
  );
  const locations = [];
  const { queryClient: cache } = createAuthenticationClient(
    identityKey,
    true,
    "/admin",
    (path) => locations.push(path),
    "admin",
  );
  cache.setQueryData(identityKey, { id: "admin" });
  const mutation = cache.getMutationCache().build(cache, {
    mutationFn: () =>
      Promise.reject(
        failure("resetGrants.revoke", "DOMAIN_REASON_INVALID_SESSION"),
      ),
  });
  await assert.rejects(mutation.execute(void 0));
  assert.deepEqual(locations, ["/admin/login"]);
  assert.equal(cache.getQueryData(identityKey), null);
  cache.clear();
});
