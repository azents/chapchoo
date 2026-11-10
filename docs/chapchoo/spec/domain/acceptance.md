---
title: "Verification layers and release evidence"
---

# Verification layers and release evidence

Each test layer establishes a specific owned contract. Synthetic SDK bytes, real Engine acceptance, browser journeys and live cluster rollout are different evidence.

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
A separate `@chapchoo/e2e test:conversation` lane requires explicit disposable
Docker inputs and composes production web listeners with actual backend,
Provider, Connector and the official ACP adapter. An execution-owned deterministic
app-server executable is selected through the existing-CLI capability. Its fixed
synthetic login/output/request never calls a model. The representative journey
covers native text, an update to the same utterance, a distinct second utterance,
an explicit typed inline answer and retained state on browser reentry. Missing
native prerequisites fail. This is deterministic assembled-plumbing coverage,
not authenticated Codex/model or general compatibility acceptance. Default
browser scenarios and browser-free native integration remain separate owners.
Web-core separately owns the actual logout-response HTTP/browser integration.
Browser delayed-cookie arrival semantics are dependency behavior and are not a
product acceptance scenario.

The reusable native package workflow assembles images on actual x86-64/ARM64 runners,
runs the independent-image smoke check on those existing local images, and
exports archives/checksums. This checks root/prefixed startup, runtime isolation,
static assets, private RPC and shutdown before PR merge without another build
or product E2E. General CI owns product-test orchestration:
its single downstream `browser-e2e` job waits only for the x86-64 package and consumes
that same run's x86-64 archive, verifies its checksum, and runs the four core
PostgreSQL/browser journeys once using only the bundled executable, Node and web
outputs. It does not rebuild the product or repeat source/per-architecture E2E.
Each architecture's package waits only for its own web and binary producers.
ARM64 remains a native build/ELF/assembly/image-smoke gate rather than a second
product-test lane. Snapshot checks the exact images it publishes as well.
Workflow configuration is not an executed success; actual exact-head CI results
must be reported separately.

The separate required `docker-integration` job owns a browser-free real Docker
lifecycle. It downloads that same run's native binary artifact for actual
backend/Control/Provider and the Computer image; no web artifact, server or
Chromium is required. It starts after the x86-64 binary producer, independently
of web builds and native packaging. `typescript/apps/integration` calls generated public
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
product work or prunes the daemon. A lost fixture-canary Create response permits
only exact reserved-name/run-label inspection and individually verified cleanup;
unknown Create outcomes remain failed even after observed cleanup. This does not
reconstruct product authority or replay Create/Start. Bounded native evidence is published only for
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

General CI runs repository checks, frontend read-only quality, and frontend
unit/Storybook/component tests as independent jobs alongside Rust quality and
native builds. The `validate` aggregate succeeds only when all three validation
owners succeed; failures, cancellation and skipped owners fail the aggregate.
The frontend test owner explicitly builds shared RPC exports before direct unit
tests rather than relying on the repository hooks or static checks to build them.
The repository-hook step omits only the separately owned quality calls;
generated RPC and other contract hooks still execute. Local commit hooks retain
their original full checks. Cargo, pre-commit environments, Turbo results and
Next compiler caches are reused with toolchain/dependency-aware keys; source
verification and actual tests remain required on every full CI execution.

No live Helm/Argo deployment, external TLS/HTTP2 or live product acceptance is
established yet. Those gates require explicit cluster/namespace, exposure/TLS,
images, PostgreSQL data/retention, existing fixed-key Secret references and Argo
full-sync scope. Requirements/Design implementation state and plan cleanup remain
pending until the complete approved acceptance is achieved.

## Connections

- [Linux artifacts and image assembly](account-delivery.md) — Build owners produce deployment-neutral inputs; packaging validates and assembles those exact artifacts without rebuilding them.
- [Helm and Argo deployment boundaries](deployment.md) — The public chart composes workloads and explicit prerequisites; offline rendering does not prove a live rollout or substrate cleanup.
- [Snapshot publication and downstream handoff](snapshot-publication.md) — A Snapshot publication binds architecture artifacts, image digests and source/run identity. Downstream deployment owns its own activation and data prerequisites.
- [Docker connection, volume and container contracts](computer-docker.md) — The Docker adapter owns one selected source and exact native resources, with uncertainty distinct from absence or confirmed cleanup.
- [Kubernetes resource and retirement contracts](computer-kubernetes.md) — Supported Kubernetes SDK operations validate original resource identities and ordered terminal evidence, without equating a delete response with retirement.
- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.

## References

- [`.github/workflows/ci.yaml`](../../../../.github/workflows/ci.yaml)
- [`typescript/apps/e2e`](../../../../typescript/apps/e2e)
- [`typescript/apps/integration`](../../../../typescript/apps/integration)
- [`testenv/chapchoo`](../../../../testenv/chapchoo)
