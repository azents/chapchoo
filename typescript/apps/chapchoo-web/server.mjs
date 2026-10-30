import { createServer } from "node:http";
import { fileURLToPath } from "node:url";
import next from "next";
import { getConfig, getListener } from "./src/config/index.ts";

// The public Next server API runs one deployment-neutral compiled application.
// Only this process owns the configured public mount; no generated output edits.
const directory = fileURLToPath(new URL(".", import.meta.url));
const configuration = getConfig();
const listener = getListener();
const app = next({
  dev: false,
  dir: directory,
  hostname: listener.host,
  port: listener.port,
});
await app.prepare();
const handle = app.getRequestHandler();
const prefix = configuration.basePath;
const server = createServer((request, response) => {
  const raw = request.url ?? "/";
  const boundary = raw.indexOf("?");
  const pathname = boundary < 0 ? raw : raw.slice(0, boundary);
  if (prefix && pathname !== prefix && !pathname.startsWith(`${prefix}/`)) {
    response.writeHead(404).end();
    return;
  }
  // Canonicalize the public request before Next sees a mount-relative URL.
  // This keeps framework trailing/duplicate-slash redirects inside the mount.
  const canonical =
    pathname
      .replaceAll("\\", "/")
      .replace(/\/{2,}/g, "/")
      .replace(/\/$/, "") || "/";
  if (prefix && canonical !== pathname) {
    const query = boundary < 0 ? "" : raw.slice(boundary);
    response.writeHead(308, { Location: `${canonical}${query}` }).end();
    return;
  }
  request.url = prefix ? raw.slice(prefix.length) || "/" : raw;
  if (request.url.startsWith("?")) request.url = `/${request.url}`;
  Promise.resolve(handle(request, response)).catch(() => {
    if (!response.headersSent) response.writeHead(500);
    response.end();
  });
});
await new Promise((resolve, reject) => {
  server.once("error", reject);
  server.listen(listener.port, listener.host, resolve);
});
console.log("Ready in runtime configuration");
let closing = false;
function stop() {
  if (closing) return;
  closing = true;
  server.close(() =>
    app.close().then(() => {
      process.exitCode = 0;
    }),
  );
  server.closeIdleConnections();
}
process.on("SIGTERM", stop);
process.on("SIGINT", stop);
