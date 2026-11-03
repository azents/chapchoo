---
title: "Account Linux Distribution and Deployment Contracts"
created: 2026-10-30
tags: [accounts, packaging, deployment, cli]
spec_type: domain
domain: accounts
code_paths:
  - rust/apps/chapchoo
  - scripts/package-linux.py
  - scripts/prepare-web-artifacts.py
  - scripts/build-images.sh
  - scripts/build-web.sh
  - scripts/build-binary.sh
  - scripts/test-installed-linux.py
  - scripts/test-container-images.py
  - images/chapchoo
  - infra/charts/chapchoo
  - testenv/chapchoo
  - typescript/apps/integration
  - .github/workflows/linux-package.yaml
  - .github/workflows/build-web.yaml
  - .github/workflows/build-binary.yaml
  - .github/workflows/snapshot.yaml
  - scripts/publish-snapshot.py
last_verified_at: 2026-11-03
spec_version: 1
---

# Account Linux Distribution and Deployment Contracts

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

## Assembly and images

The account acceptance Assembly uses separate public `serve --groups client,control`
and `serve --groups admin` processes and loopback RPC listeners. General/Admin web
listeners receive only their corresponding explicit backend URL, so a missing
Admin-only contract cannot be hidden by a combined union. Secondary-user seeding
uses the separate current Client/Admin endpoints. Its private SWC cache is owned by the fixture;
Next build receives tooling settings, not deployment backend/public configuration.

`scripts/package-linux.py` accepts explicit built inputs; it neither builds nor
downloads. It validates matching ELF64 architecture, pinned official Node24.22.1
inventory/hash/bytes, safe links and app selection. Credential/state paths and
escaping/broken/cyclic links fail. Archive and SHA-256 sidecar are deterministic
for the input/epoch and are published without overwriting an existing output.
The standard gzip stream uses fast compression rather than spending CI time on
the maximum compression level; archive contents and validation are unchanged.
The distribution includes the project and Node licenses. Linux x86-64 and ARM64
are the supported targets; the release binary uses the fixed Bookworm builder.
Deployment hosts need no Node, pnpm, Turbo or build toolchain.

`images/chapchoo/Dockerfile` has independent backend/general-web/admin-web targets
from digest-pinned multiarchitecture official images. Separate build jobs own
Rust/vendored-protoc and Next compilation. Image assembly only validates and
copies their outputs; final images do not contain build toolchains. Runtime UID/GID is10001. Backend has no Node/web;
web has the central executable, matching Node and selected standalone app, and
receives no backend database/key credentials. Runtime supports a read-only root
with an explicitly writable temporary directory. Database state is separately
operator-provided, never baked into an image.

The standard runtime stages `/workspace` owned by UID:GID 10001:10001. A true
initial fresh Docker volume can copy that directory's ownership; existing
original volumes use no-copy. The runtime never uses a privileged ownership
repair helper. Custom mounts/images must meet the same non-root write contract.

## Chart and controller lifecycle

`infra/charts/chapchoo` configures a combined Client/Admin/Control union or
independently scalable group deployments, independent webs and private ClusterIP
RPC Services. General/Admin service ports 50051/50052 target the selected named
gRPC listener; Control has its own Service. Network peer 50053 is a separate
internal-only listener/Service, never a public Ingress. One external PostgreSQL
authority and preexisting Secret references are required.
The chart does not create database/key Secrets or provision a database. Web
configuration contains selected listener/public/backend settings and optional
peer public web navigation metadata. Public URL
paths directly determine Ingress and health paths; images contain no selected
deployment path. Artifact execution, not a declaration, verifies portability.

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

### Computer component deployment

The peer ingress NetworkPolicy permits only this release's backend server
workloads on 50053; actual CNI denial/enforcement is a live release gate, not a
rendering result. Component/public gRPC remains a separate authenticated boundary.
A scoped existing system-owner Secret is provisioned only to Control and the
selected Computer Provider, not webs, separate Client/Admin or execution Pods.

The optional Computer Provider is independent of Control replicas, defaults to
one replica/Recreate, and uses the binary-only image with explicit runtime image,
resource/storage and namespace policy. Its managed namespace must already exist
and be explicitly selected. Namespaced Role/Binding grants Pod get/list/watch/
create/delete and PVC/Secret get/create/delete, with a distinct token-free
execution ServiceAccount. No product DB/root environment is provisioned to this
process. Namespace Secret get still grants visibility to all Secrets in that
namespace; operators must isolate managed resources from backend credentials.
Default replica count is not quiescence proof. Bootstrap credentials are created
through the actual Provider's acknowledged resource flow, not Helm templates.

See [Computer Control RPC](computer-control-rpc.md),
[Kubernetes boundaries](computer-kubernetes.md) and
[owner/operator web](computer-web.md). Live resource retirement, original PVC
contents, policy enforcement and component TLS require approved disposable
prerequisites and are not established by offline/synthetic tests.

## Snapshot publication and Home handoff

The `Snapshot` workflow follows the requested Azents-style image publication:
main pushes or explicit main dispatch build on native x86-64/ARM64 runners.
Each runner builds independent backend/General/Admin images, exercises their
nonroot/read-only/private-RPC/static/shutdown boundary and only then pushes
architecture staging images. All builds are deployment-neutral; test execution
selects root and non-root paths at runtime. Snapshot is a development distribution
channel, not a Home build variant. A release of the same commit must promote
the same verified artifact/digest rather than rebuild an application variant.

After both native jobs succeed, publication validates exactly six matching
component/architecture metadata files, creates three multiarchitecture indexes
and checks their registry digest and exact Linux platform set. Immutable tags
include source SHA, run and attempt; only a complete set gets a
`snapshot-payload` artifact. An optional downstream dispatch carries only run
identity. Operators configure `DOWNSTREAM_DEPLOY_REPOSITORY` and
`DOWNSTREAM_DEPLOY_TOKEN`; missing downstream configuration is reported without
pretending a Home update occurred.

Home owns PostgreSQL/PVC/ExternalSecret prerequisites and private Traefik Gateway
routes separately from this chart. Its updater verifies the successful canonical
main Snapshot and downloads that run's artifact before atomically pinning chart
SHA and all images. It does not trust image data in a dispatch notification or
infer that Secrets are ready. Initial activation and live acceptance remain
explicit; no new nginx controller or public plaintext RPC is introduced.

## Verification boundary

Local source verification covers safe assembly and offline chart contracts,
SQLite/PostgreSQL domain/RPC checks and four assembled PostgreSQL/browser journeys
(three account/team and one Computer owner/operator). App Storybooks/component
checks cover detailed copy/status/mobile behavior. The Computer fixture runs
actual processes over synthetic supported-client HTTP, not a real Kubernetes
cluster. New image/native-architecture acceptance belongs to exact-head CI and
must be reported separately rather than inferred from source checks. Lower Rust
lifecycle tests include partial startup, foreign listener isolation and
resistant-descendant cleanup.
Web-core separately owns the actual logout-response HTTP/browser integration.
Browser delayed-cookie arrival semantics are dependency behavior and are not a
product acceptance scenario.

The reusable native build workflow builds images on actual x86-64/ARM64 runners,
runs the independent-image smoke check on those existing local images, and
exports archives/checksums. This checks root/prefixed startup, runtime isolation,
static assets, private RPC and shutdown before PR merge without another build
or product E2E. General CI owns product-test orchestration:
its single downstream `browser-e2e` job waits for the native build and consumes
that same run's x86-64 archive, verifies its checksum, and runs the four core
PostgreSQL/browser journeys once using only the bundled executable, Node and web
outputs. It does not rebuild the product or repeat source/per-architecture E2E.
ARM64 remains a native build/ELF/assembly/image-smoke gate rather than a second
product-test lane. Snapshot checks the exact images it publishes as well.
Workflow configuration is not an executed success; actual exact-head CI results
must be reported separately.

The separate required `docker-integration` job owns a browser-free real Docker
lifecycle. It downloads that same run's native binary artifact for actual
backend/Control/Provider and the Computer image; no web artifact, server or
Chromium is required. `typescript/apps/integration` calls generated public
account/Computer RPC and asserts connected Create, Stop/Start with original data,
successor no-copy including emptied storage, normal Delete and untouched canaries.
Public Docker SDK support supplies only prerequisites/marker IO. Existing browser
and synthetic-Kubernetes coverage stays unchanged. Missing required native
socket/image/route fails, never skips. Safe startup diagnostics distinguish
static native error classification and RPC status without raw source messages.

Only this opt-in real-Docker Assembly binds ephemeral Client/Control to all
interfaces so the existing bridge gateway (or explicit host override) can route
the Connector. Ordinary fixtures remain loopback-bound. No guest port, host
networking or tunnel is introduced. Test teardown never force-deletes ambiguous
product work or prunes the daemon. Bounded native evidence is published only for
a passed test; raw guest/bootstrap output remains private.

Separate reusable web and binary build jobs produce architecture-matched
artifacts. The web job builds Next on a native Linux runner and exports validated
standalone/static/public inputs; the binary job uses the pinned Bookworm Rust
build environment. Same-run artifact transfer preserves hidden Next files and
symlinks inside tar files. Image and archive jobs both consume those outputs.
`images/chapchoo/Dockerfile` is packaging-only: named `binary-artifacts` and
`web-artifacts` contexts are required and it never runs Cargo, Next or pnpm.
`ci-bake.hcl` assembles three independent native images using those contexts.

Local `bash scripts/build-images.sh` first runs the same target builds through
separate Linux builder containers, then packages the outputs. On Apple Silicon
it defaults to Linux ARM64; Mac node_modules never enter Linux artifacts. Local
builder cache mounts are target-scoped. CI Cargo, web compiler/Turbo and packaging
layer caches are separated by their real toolchain/dependency/architecture
inputs. Cache restoration never replaces Rust SQLite/PostgreSQL tests, the single
browser gate, archive/checksum checks or Snapshot's image execution checks.

General CI assigns Rust quality to its Rust job and frontend quality to explicit
read-only checks. The repository-hook step omits only those duplicated calls;
generated RPC and other contract hooks still execute. Local commit hooks retain
their original full checks. Cargo, pre-commit environments, Turbo results and
Next compiler caches are reused with toolchain/dependency-aware keys; source
verification and actual tests remain required on every full CI execution.

No live Helm/Argo deployment, external TLS/HTTP2 or live product acceptance is
established yet. Those gates require explicit cluster/namespace, exposure/TLS,
images, PostgreSQL data/retention, existing fixed-key Secret references and Argo
full-sync scope. Requirements/Design implementation state and plan cleanup remain
pending until the complete approved acceptance is achieved.

## Deployment secret clarification

`CHAPCHOO_SECRET_KEY` and chart `secretKey` replace the prior configuration name
without a fallback alias. Preserve the existing key bytes; crypto derivation and
DB binding are unchanged. Home's PASSWORD is the sole external DB secret input;
its ExternalSecret template creates both password and escaped postgres-url.

The default chart also owns first-admin proof bootstrap (Design37/M44), distinct
from schema preparation. Hook SA/Role/Binding at-30 authorize a finite Job at-20;
prepare remains-10. Bootstrap uses only namespace/name/field and a projected API
token, never the DB/root key. It gets the one named Secret, creates if absent,
validates/reuses existing or concurrently created material and never overwrites.
RBAC grants named get and namespace Secret create, not list/update/patch/delete;
create cannot be restricted to one resourceName by Kubernetes RBAC. Backend/web
pods and schema prepare retain no token; the Computer Provider uses its own
namespaced authority described above. Secret values do not appear in rendered
manifests/logs and survive hook/release deletion. Explicit existing setup Secret
configuration skips bootstrap. The backend image includes a pinned official
kubectl1.35 client; compatible Kubernetes server version is required.

The setup proof is not an Infisical prerequisite in Home. Authorized operators
retrieve it privately after bootstrap for the first Admin setup; DB consumption
still prevents reinitialization. Live bootstrap/API authorization and deployment
remain separately unexecuted until publication and activation approval.
