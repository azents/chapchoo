import assert from "node:assert/strict";
import test from "node:test";
import { status } from "@grpc/grpc-js";
import { logoutResponse } from "../dist/logout.js";
import { cookiePolicy } from "../dist/cookies.js";
import { webConfig } from "../dist/config.js";
import { RpcFailure } from "../dist/errors.js";

function config(app) {
  return webConfig(app, {
    publicUrl: `http://127.0.0.1:3003/${app === "admin" ? "admin" : "app"}`,
    bindHost: "127.0.0.1",
    backendUrl: "http://127.0.0.1:50051",
  });
}
function request(
  c,
  origin = c.origin,
  token = "x".repeat(43),
  method = "POST",
) {
  return new Request(`${c.publicUrl}/api/logout`, {
    method,
    headers: { origin, cookie: `${cookiePolicy(c).name}=${token}` },
  });
}

test("both apps immediately dispose their cookie before best-effort backend revocation", async () => {
  const previous = console.error;
  const logs = [];
  console.error = (line) => logs.push(line);
  try {
    for (const app of ["general", "admin"]) {
      const c = config(app);
      let calls = 0;
      const tasks = [];
      const response = logoutResponse(
        request(c),
        c,
        () => {
          calls += 1;
          return Promise.reject(
            new RpcFailure(
              status.UNAVAILABLE,
              null,
              new Error("private backend detail"),
            ),
          );
        },
        (task) => tasks.push(task),
      );
      assert.equal(calls, 0);
      assert.equal(tasks.length, 1);
      assert.equal(response.status, 303);
      assert.equal(
        response.headers.get("location"),
        `${c.origin}${c.basePath}/login`,
      );
      assert.equal(response.headers.get("cache-control"), "no-store");
      const cookie = response.headers.get("set-cookie");
      assert.ok(cookie.startsWith(`${cookiePolicy(c).name}=;`));
      assert.match(cookie, /HttpOnly/);
      assert.match(cookie, /Max-Age=0/);
      assert.ok(!cookie.includes("x".repeat(43)));
      assert.match(cookie, app === "admin" ? /Path=\/admin;/ : /Path=\/;/);
      await tasks[0]();
      assert.equal(calls, 1);
    }
    assert.equal(logs.length, 2);
    assert.ok(logs.every((line) => !line.includes("private backend detail")));
  } finally {
    console.error = previous;
  }
});

test("logout admits only same-origin POST before any credential work", () => {
  const c = config("general");
  const unexpected = () => {
    throw new Error("Unadmitted request performed credential work");
  };
  for (const [origin, method, expected] of [
    ["https://other.invalid", "POST", 403],
    ["", "POST", 403],
    [c.origin, "GET", 405],
  ]) {
    const response = logoutResponse(
      request(c, origin, "x".repeat(43), method),
      c,
      unexpected,
      unexpected,
    );
    assert.equal(response.status, expected);
    assert.equal(response.headers.get("set-cookie"), null);
  }
});

test("logout without a valid presented cookie remains a local idempotent disposal", () => {
  const c = config("admin");
  const unexpected = () => {
    throw new Error("Invalid cookie scheduled revocation");
  };
  for (const token of ["invalid", ""]) {
    const response = logoutResponse(
      request(c, c.origin, token),
      c,
      unexpected,
      unexpected,
    );
    assert.equal(response.status, 303);
    assert.match(response.headers.get("set-cookie"), /Max-Age=0/);
  }
});
