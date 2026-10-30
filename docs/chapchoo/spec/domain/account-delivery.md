---
title: "Account Linux Distribution and Deployment Contracts"
created: 2026-10-30
tags: [accounts, packaging, deployment, cli]
spec_type: domain
domain: accounts
code_paths:
  - rust/apps/chapchoo
  - scripts/package-linux.py
  - scripts/test-installed-linux.py
  - scripts/test-container-images.py
  - images/chapchoo
  - infra/charts/chapchoo
  - testenv/chapchoo
  - .github/workflows/linux-package.yaml
last_verified_at: 2026-10-30
spec_version: 1
---

# Account Linux Distribution and Deployment Contracts

## Foreground execution

`chapchoo serve --role general|admin|both|none` selects backend listeners;
optional `--web general|admin|both` selects web children. Selecting nothing fails.
Web-only execution does not load database, installation-key or setup configuration.
Operator `prepare` and `grant-admin` retain their DB/key authority. There is no
host Node fallback, daemon, instance registry, management IPC or restart loop.

The executable resolves its own installation directory. Archives contain:

```text
chapchoo
runtime/bin/node
web/general/apps/chapchoo-web/server.js
web/admin/apps/chapchoo-admin-web/server.js
```

The corresponding app roots contain public/static assets; each traced tree has
its own dependencies. Web images include only their selected app. Explicit
`CHAPCHOO_GENERAL_WEB_LISTEN` / `CHAPCHOO_ADMIN_WEB_LISTEN` (and CLI equivalents)
default to 127.0.0.1:3003/3004. `CHAPCHOO_GENERAL_WEB_URL` /
`CHAPCHOO_ADMIN_WEB_URL` configure public origins, including issued general links.
`CHAPCHOO_GENERAL_BACKEND_URL` / `CHAPCHOO_ADMIN_BACKEND_URL` configure RPC clients.
A co-selected wildcard backend bind maps to loopback; specific binds keep their
address. Independently hosted web defaults to local50051/50052. Public HTTP
requires actual loopback binding; external origins require HTTPS. Compiled and
runtime public paths must match; runtime prefix rewriting is not provided.

Child environment is an allowlist of runtime/locale/CA settings and the selected
web URL/listener/backend configuration. It excludes database/root/setup/AWS
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

## Assembly and images

`scripts/package-linux.py` accepts explicit built inputs; it neither builds nor
downloads. It validates matching ELF64 architecture, pinned official Node24.22.1
inventory/hash/bytes, safe links and app selection. Credential/state paths and
escaping/broken/cyclic links fail. Archive and SHA-256 sidecar are deterministic
for the input/epoch and are published without overwriting an existing output.
The distribution includes the project and Node licenses. Linux x86-64 and ARM64
are the supported targets; the release binary uses the fixed Bookworm builder.
Deployment hosts need no Node, pnpm, Turbo or build toolchain.

`images/chapchoo/Dockerfile` has independent backend/general-web/admin-web targets
from digest-pinned multiarchitecture official images. Build stages retain the
actual Rust/vendored-protoc and Node generation prerequisites; final images do
not contain build toolchains. Runtime UID/GID is10001. Backend has no Node/web;
web has the central executable, matching Node and selected standalone app, and
receives no backend database/key credentials. Runtime supports a read-only root
with an explicitly writable temporary directory. Database state is separately
operator-provided, never baked into an image.

## Chart and controller lifecycle

`infra/charts/chapchoo` configures combined or independent backend deployments,
independent web deployments and distinct private ClusterIP RPC Services. One
external PostgreSQL authority and preexisting Secret references are required.
The chart does not create database/key Secrets or provision a database. Web
configuration contains only selected listener/public/backend settings. Image
`compiledBasePath` declarations must agree with URL paths, but declarations alone
are not proof of the image's compiled configuration; runtime health validates it.

A single fixed-name prepare Job uses both native Helm pre-install/pre-upgrade
weight-10 and Argo PreSync wave-10 annotations. It depends only on preexisting
DB/key Secrets, not later ConfigMaps, ServiceAccounts or same-app Sync Secrets.
It has a finite deadline, no retries/restarts or service-account token, and
before-creation/success cleanup; failure evidence is retained. Applications do
not run migrations. Termination grace is at least30s. Argo acceptance requires a
full application sync: selective resource sync does not execute hooks.

Optional ingress-nginx external RPC edges require explicit TLS/HTTP2 settings
and distinct General/Admin hosts. Internal plaintext is a trusted network hop,
not end-to-end TLS. Public web URLs require HTTPS. A rendered Ingress is not
proof of certificate, metadata/trailer or product behavior. Image rollback does
not roll back database schema or committed data; arbitrary zero downtime is not
promised. Examples under `testenv/chapchoo` do not authorize a cluster write.
The chart rejects a gRPC catch-all sharing the same ingress class, host and root
path with an enabled web route. Different web prefixes remain valid, including
General `/` and Admin `/admin` on one host.

## Verification boundary

Root local x86-64 verification covers safe assembly14 tests; chart lint and11
contract tests including35 invalid configurations; the extracted official Node;
SQLite preparation and both bundled apps/static/fonts with SIGINT and closed
owned listeners; all three PostgreSQL account/team browser journeys plus the
focused delayed HttpOnly-cookie test; and independent nonroot/read-only images,
private RPC routing and graceful stop. Lower Rust lifecycle tests include partial
startup, foreign listener isolation and resistant-descendant cleanup.

Native Linux CI runs the same archive checks and PostgreSQL journeys on actual
x86-64/ARM64 runners and exercises the independent images before uploading only
verified archives/checksums. Workflow configuration is not an executed success;
actual exact-head CI results must be reported separately.

No live Helm/Argo deployment, external TLS/HTTP2 or live product acceptance is
established yet. Those gates require explicit cluster/namespace, exposure/TLS,
images, PostgreSQL data/retention, existing fixed-key Secret references and Argo
full-sync scope. Requirements/Design implementation state and plan cleanup remain
pending until the complete approved acceptance is achieved.
