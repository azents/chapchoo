import assert from "node:assert/strict";
import test from "node:test";
import { refreshAdminState } from "./refreshAdminState.ts";

void test("explicit Admin recovery retries identity as well as both grant inventories", async () => {
  const refreshed: string[] = [];
  function query(name: string): { invalidate: () => Promise<void> } {
    return {
      invalidate: () => {
        refreshed.push(name);
        return Promise.resolve();
      },
    };
  }
  await refreshAdminState({
    auth: { identity: query("identity") },
    signupGrants: { list: query("signup") },
    resetGrants: { list: query("reset") },
  });
  assert.deepEqual(refreshed, ["identity", "signup", "reset"]);
});

void test("explicit Admin recovery does not hide a remaining identity failure", async () => {
  await assert.rejects(
    refreshAdminState({
      auth: {
        identity: {
          invalidate: () =>
            Promise.reject(new Error("Identity still unavailable")),
        },
      },
      signupGrants: { list: { invalidate: () => Promise.resolve() } },
      resetGrants: { list: { invalidate: () => Promise.resolve() } },
    }),
    /Identity still unavailable/,
  );
});
