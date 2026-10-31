import { createServer } from "node:http";
import { webConfig } from "@chapchoo/web-core/config";
import { logoutResponse } from "@chapchoo/web-core/logout";
import { cookiePolicy, sessionCookie } from "@chapchoo/web-core/cookies";
import { expect, test } from "@playwright/test";
import { randomBytes } from "node:crypto";

function deferred() {
  let resolve;
  const promise = new Promise((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}

// Exercise the application logout response over real HTTP, not browser cookie ordering.
test("logout disposes only its app cookie despite failed backend revocation", async ({
  browser,
}) => {
  const current = randomBytes(32).toString("base64url");
  let origin = "";
  let logoutAttempts = 0;
  const revocation = deferred();
  function respond(response, cookie) {
    response.writeHead(200, {
      "Content-Type": "application/json",
      "Cache-Control": "no-store",
      "Set-Cookie": cookie,
    });
    response.end("{}");
  }
  function policy() {
    return cookiePolicy(
      webConfig("general", {
        publicUrl: origin,
        bindHost: "127.0.0.1",
        backendUrl: "http://127.0.0.1:1",
      }),
    );
  }
  const server = createServer((request, response) => {
    if (request.url === "/newer") {
      respond(
        response,
        sessionCookie(policy(), current, Date.now() + 60_000, Date.now()),
      );
    } else if (request.url === "/newer-admin") {
      const admin = cookiePolicy(
        webConfig("admin", {
          publicUrl: `${origin}/admin`,
          bindHost: "127.0.0.1",
          backendUrl: "http://127.0.0.1:1",
        }),
      );
      respond(
        response,
        sessionCookie(admin, current, Date.now() + 60_000, Date.now()),
      );
    } else if (request.url === "/logout-failure") {
      const config = webConfig("general", {
        publicUrl: origin,
        bindHost: "127.0.0.1",
        backendUrl: "http://127.0.0.1:1",
      });
      const result = logoutResponse(
        new Request(`${origin}/logout-failure`, {
          method: request.method,
          headers: {
            origin: request.headers.origin ?? "",
            cookie: request.headers.cookie ?? "",
          },
        }),
        config,
        () => {
          logoutAttempts += 1;
          return Promise.reject(new Error("Backend unavailable"));
        },
        (task) =>
          response.once("finish", () => {
            void task().then(() => revocation.resolve());
          }),
      );
      response.writeHead(result.status, Object.fromEntries(result.headers));
      response.end();
    } else {
      response.writeHead(200, { "Content-Type": "text/html" });
      response.end("<!doctype html><title>Owned cookie boundary</title>");
    }
  });
  const context = await browser.newContext();
  try {
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(0, "127.0.0.1", resolve);
    });
    const address = server.address();
    if (!address || typeof address === "string") {
      throw new Error("The owned cookie listener has no allocated address");
    }
    origin = `http://127.0.0.1:${address.port}`;
    const page = await context.newPage();
    await page.goto(origin);
    await page.evaluate(async () => {
      await fetch("/newer", { method: "POST" });
      await fetch("/newer-admin", { method: "POST" });
      const form = document.createElement("form");
      form.action = "/logout-failure";
      form.method = "POST";
      document.body.append(form);
      form.submit();
    });
    await page.waitForURL(`${origin}/login`);
    await revocation.promise;
    const remaining = await context.cookies(`${origin}/admin`);
    expect(
      remaining.some((cookie) => cookie.name === "chapchoo-general-session"),
      "Backend failure must not retain the locally signed-out cookie",
    ).toBe(false);
    expect(
      remaining.some((cookie) => cookie.name === "chapchoo-admin-session"),
      "General logout must preserve the independent Admin cookie",
    ).toBe(true);
    expect(
      logoutAttempts,
      "Logout revocation must not be automatically replayed",
    ).toBe(1);
  } finally {
    await context.close();
    server.closeAllConnections();
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()));
    });
  }
});
