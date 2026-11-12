---
title: "Foreground process ownership and health"
---

# Foreground process ownership and health

The composition root owns admission, startup and shutdown of its selected backend and web processes. A successful cleanup submission is not completed shutdown.

## Foreground lifecycle and health

The selected group union defaults to loopback plaintext HTTP/2 on port 50051. Explicit
deployment binds require a trusted internal network and a separate operator TLS
edge for external access. The executable does not issue certificates or provide
native gRPC TLS.

Selected backend groups and optional bundled web children are one failure unit.
Readiness follows configuration, read-only
DB/schema/key checks, dependency construction and successful binds. Partial bind
failure drops already acquired sockets. SIGINT/SIGTERM withdraws admission before
drain. Owned tasks are joined; a component exit or cleanup timeout returns failure.
Defaults are 60 seconds for startup, 20 for graceful shutdown and a further
5 for forced cleanup. Started blocking work is not assumed abortable.
Backend task, web reap and signal failures are retained together privately instead
of replacing an earlier failure with a later cleanup result. Completed forced
cleanup still reports failure rather than a successful graceful stop.

The Client group owns device acquisition, while Control owns the separate
[execution authentication service](backend-authentication.md#component-credential-delivery).
Their fixed HTTPS provider collaborators and effect owners are composed before
listener admission. Started provider calls belong to bounded task owners, not
browser/RPC waiters. Shutdown closes both admissions and joins their work alongside
backend/web cleanup. Both owners are retained across partial startup failure and
cancelled drain futures, including already-collected causes. Execution
authentication cleanup failures remain combined with backend and other cleanup
failures. A cleanup deadline is not proof that an uncertain provider effect was
rolled back.

Control also composes the bounded [server-tools effect owner](server-tools.md)
before listener admission. Shutdown closes its admission and joins component
forwarding plus started effects alongside the other cleanup owners. A cancelled
drain waiter retains owned tasks; timeout does not turn possible publication
into known nonexecution. Cleanup failures remain combined with the backend cause.

Unary gRPC health for an empty service, either common service or any domain
service actually registered on the listener rereads read-only database
readiness with bounded probe capacity. `chapchoo.liveness` reports process
responsiveness independently. Unknown services return NOT_FOUND; Watch is not
implemented. Optional `--web general|admin|both` starts executable-relative bundled
Node children; `--groups` without values permits web-only execution without loading DB/key
configuration. There is no daemon, automatic restart or global process scan.
See [Linux delivery](account-delivery.md) for child ownership and package layout.

## Foreground execution

`chapchoo serve --groups client,admin,control` selects a backend service union;
optional `--web general|admin|both` selects web children. Selecting nothing fails.
Web-only execution does not load database, secret-key or setup configuration.
Operator `prepare` and `grant-admin` retain their DB/key authority. There is no
host Node fallback, daemon, web-child management IPC or restart loop. Control's
product DB directory and separate peer listener are described in
[Computer Control RPC](computer-control-rpc.md).

Explicit preparation also atomically initializes display defaults and upgrades
existing users. The first successful preparation machine supplies initial time
zone and supported locale; later default edits and repreparation preserve
established values. Missing existing-user time zones use the migration machine.
Bundled timezone validation data keeps runtime packages independent of host
tzdata installation. `serve`/health only verify complete preparation; deploying a
new image does not authorize startup migrations.

The executable resolves its own installation directory. Archives contain:

```text
chapchoo
runtime/bin/node
web/general/typescript/apps/web/server.mjs
web/admin/typescript/apps/admin-web/server.mjs
```

The corresponding app roots contain public/static assets; each traced tree has
its own dependencies. Web images include only their selected app. Explicit
`CHAPCHOO_GENERAL_WEB_LISTEN` / `CHAPCHOO_ADMIN_WEB_LISTEN` (and CLI equivalents)
default to 127.0.0.1:3003/3004. `CHAPCHOO_GENERAL_WEB_URL` /
`CHAPCHOO_ADMIN_WEB_URL` configure public origins, including issued general links.
`CHAPCHOO_GENERAL_BACKEND_URL` / `CHAPCHOO_ADMIN_BACKEND_URL` configure RPC clients.
Each selected web application requires its explicit backend URL; neither target
is inferred from a local bind or another web application's target. Public HTTP
requires actual loopback binding; external origins require HTTPS. Public paths
are runtime-only. The public Next custom server accepts only its exact mount
boundary, then dispatches the mount-relative request to Next without editing
generated HTML or JavaScript.

Child environment is an allowlist of runtime/locale/CA settings, the selected
web URL/listener/backend configuration and explicitly configured peer public web
URL for navigation. The peer URL is optional, validated and never supplies an API
destination. When both webs are selected, each uses the other selected app's
resolved public URL, including supported local defaults; a standalone app needs
explicit peer metadata. An Admin child receives no General
backend target. It excludes database/root/setup/AWS
credentials, Node preloads and TLS-verification disabling. Each child uses the
absolute bundled Node path and its own process group. Admission waits for every
selected backend and each owned web child's Ready banner plus HTTP health.
An unrelated HTTP listener cannot satisfy that child's startup ownership.
The first-ready web child's exit remains supervised while the other starts;
both web children are checked again before enabling common admission.

Signal, failed startup or unexpected component exit withdraws admission and stops
the selected invocation as one failure unit. Startup is bounded to60s, graceful
cleanup20s and forced cleanup a further5s. All child/readers remain tracked;
TERM/KILL and waits apply only to owned groups. Linux invocation-local subreaping
adopts orphaned descendants, and a non-reaping owned-child check fences group-ID
reuse before signaling. There is no global process-table scan or persistent
ownership registry. Cleanup failure returns failure and retains private sources;
it does not assert rollback of already committed work.

## Connections

- [Identity-cutover operator procedure](repository-identity.md#operator-procedure-for-the-identity-cutover) — coordinated maintenance owns quiescence, original saved-resource evidence and paired backup; process shutdown alone proves none of those.
- [Account RPC boundaries](account-rpc.md) — Account transport owns service admission, caller credentials and validated wire outcomes. Business transactions and process lifetime have separate owners.
- [Linux artifacts and image assembly](account-delivery.md) — Build owners produce deployment-neutral inputs; packaging validates and assembles those exact artifacts without rebuilding them.
- [Helm and Argo deployment boundaries](deployment.md) — The public chart composes workloads and explicit prerequisites; offline rendering does not prove a live rollout or substrate cleanup.
- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.

## References

- [`rust/apps/chapchoo/src/main.rs`](../../../../rust/apps/chapchoo/src/main.rs)
- [`rust/apps/chapchoo/src/runtime.rs`](../../../../rust/apps/chapchoo/src/runtime.rs)
- [`rust/apps/chapchoo/src/web.rs`](../../../../rust/apps/chapchoo/src/web.rs)
- [`rust/apps/chapchoo/src/health.rs`](../../../../rust/apps/chapchoo/src/health.rs)
