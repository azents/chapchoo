import { createServer } from "node:http";
import { webConfig } from "@chapchoo/web-core/config";
import { logoutResponse } from "@chapchoo/web-core/logout";
import {
  clearSessionCookie,
  cookiePolicy,
  sessionCookie,
} from "@chapchoo/web-core/cookies";
import { expect, test } from "@playwright/test";
import { secret } from "@chapchoo/testenv/safety";
import type { ServerResponse } from "node:http";

function deferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
} {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}

// Focused browser-cookie boundary evidence, not another product journey or a
// substitute for the backend's fixed-overlap/revocation tests.
test("the browser applies delayed HttpOnly cookie responses in arrival order", async ({
  browser,
}) => {
  const older = deferred<ServerResponse>();
  const delayedClear = deferred<ServerResponse>();
  const previous = secret();
  const current = secret();
  let origin = "";
  let logoutAttempts = 0;
  const revocation = deferred<void>();
  function respond(response: ServerResponse, cookie: string): void {
    response.writeHead(200, {
      "Content-Type": "application/json",
      "Cache-Control": "no-store",
      "Set-Cookie": cookie,
    });
    response.end("{}");
  }
  function policy(): ReturnType<typeof cookiePolicy> {
    return cookiePolicy(
      webConfig("general", {
        publicUrl: origin,
        bindHost: "127.0.0.1",
        backendUrl: "http://127.0.0.1:1",
      }),
    );
  }
  const server = createServer((request, response) => {
    if (request.url === "/older") {
      older.resolve(response);
    } else if (request.url === "/delayed-clear") {
      delayedClear.resolve(response);
    } else if (request.url === "/newer") {
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
    await new Promise<void>((resolve, reject) => {
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
    const heldResponse = page.evaluate(() =>
      fetch("/older", { method: "POST" }).then((response) => response.ok),
    );
    const held = await older.promise;
    await page.evaluate(() =>
      fetch("/newer", { method: "POST" }).then(() => undefined),
    );
    expect(
      (await context.cookies(origin)).some(
        (cookie) => cookie.value === current,
      ),
      "The newer cookie must be accepted before the delayed response",
    ).toBe(true);
    respond(
      held,
      sessionCookie(policy(), previous, Date.now() + 60_000, Date.now()),
    );
    expect(await heldResponse).toBe(true);
    expect(
      (await context.cookies(origin)).some(
        (cookie) => cookie.value === previous,
      ),
      "A later-arriving old response can overwrite the newer cookie",
    ).toBe(true);
    expect(
      await page.evaluate(() =>
        document.cookie.includes("chapchoo-general-session"),
      ),
      "The HttpOnly credential must remain unavailable to document.cookie",
    ).toBe(false);

    const clearingResponse = page.evaluate(() =>
      fetch("/delayed-clear", { method: "POST" }).then(
        (response) => response.ok,
      ),
    );
    const clearing = await delayedClear.promise;
    await page.evaluate(() =>
      fetch("/newer", { method: "POST" }).then(() => undefined),
    );
    respond(clearing, clearSessionCookie(policy()));
    expect(await clearingResponse).toBe(true);
    expect(
      (await context.cookies(origin)).some(
        (cookie) => cookie.name === "chapchoo-general-session",
      ),
      "A delayed deletion can clear newer browser state; it does not revoke or restore backend authority",
    ).toBe(false);
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
    await new Promise<void>((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()));
    });
  }
});
