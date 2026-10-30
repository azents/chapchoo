---
title: "Accounts Phase 5: Linux Distribution and Deployment"
created: 2026-10-30
tags: [accounts, implementation, packaging, deployment]
---

# Phase Execution Plan

- Phase: 5/5, installed Linux CLI/runtime, independent images, chart and final acceptance.
- Branch/base: `feat/accounts-delivery` → `feat/accounts-web` at `fd9b68a` (PR #5).
- PR boundary: package the implemented account surface, extend foreground ownership
  to selected web children, provide independent deployment units and the single
  schema hook, then validate actual artifacts and the separately authorized cluster.
- Inputs: phases 1–4; generated protocol; both standalone Next builds; fixed-key
  preparation/readiness; approved Design revision 36 and exact approved mechanism set.
- Approved mechanisms: M7, M9, M28, M33, M39, M40, M42, M43; integrated
  acceptance of M1–M34/M36–M43 without introducing new authority.
- Authority: REQ-1–REQ-13 as applicable, particularly REQ-11–REQ-13;
  ADR-D3, D6, D7, D11, D13, D15–D17, D28, D30–D31, D33, D35–D37;
  current credential/storage/RPC/web Specs and Design deployment/ownership sections.
- Design delta: None.
- Deliverables: Linux x86-64/ARM64 archives with architecture-matched Node and
  both web artifacts; central foreground CLI selection and owned process groups;
  backend/general-web/Admin-web images; PostgreSQL chart with distinct Services,
  independent web units and one Helm/Argo preparation Job; artifact/lifecycle,
  chart and separately approved live account/team evidence.
- Non-goals: macOS/Windows/native apps, daemon/instance registry/management IPC,
  automatic restart, key replacement/re-encryption, replay/outbox/versioning
  systems, hosted DB/secret-provider provisioning, unrelated cluster changes.

## Shared Implementation Interfaces

These identifiers/layouts decompose the approved contract; they are not new
product or Design authority.

- Existing `serve --role general|admin|both` remains the backend selection.
  Add `--role none` only for independently selected web execution and
  `--web general|admin|both`; an empty selection fails. Operator `prepare` and
  `grant-admin` keep their existing DB/key admission. Web-only execution must
  not load a backend database or installation key.
- General/Admin web listeners use `--general-web-listen` / `--admin-web-listen`
  and corresponding `CHAPCHOO_GENERAL_WEB_LISTEN` / `CHAPCHOO_ADMIN_WEB_LISTEN`,
  defaulting to 127.0.0.1:3003/3004. Public origins use
  `CHAPCHOO_GENERAL_WEB_URL` / `CHAPCHOO_ADMIN_WEB_URL`, defaulting to
  http://localhost:3003/3004. General issued-link configuration uses the same
  explicit general public URL. No public URL is inferred from forwarded headers.
- `CHAPCHOO_GENERAL_BACKEND_URL` / `CHAPCHOO_ADMIN_BACKEND_URL` configure web
  RPC clients. Without an explicit URL, a co-selected wildcard listener maps to
  loopback while a specific bind keeps its address; independently hosted web
  defaults to local50051/50052.
  Existing Node URL/TLS verification remains authoritative; no insecure fallback.
- Archive root contains `chapchoo`, `runtime/bin/node`, and independent traced
  trees `web/general/` and `web/admin/`. Preserve Next workspace layout:
  `web/general/apps/chapchoo-web/server.js` and
  `web/admin/apps/chapchoo-admin-web/server.js`, with each app's public/static
  assets at its corresponding app root. The executable locates its own archive
  root; missing artifacts fail instead of using host Node, pnpm or Turbo.
- Use an explicit web-child environment allowlist: selected public URL, actual
  listener host/port, backend URL and required Node/runtime settings. Exclude
  DB/root/bootstrap/AWS material and Node preload options. A web-only image
  contains the central executable, its matching Node and only its selected app.
- Each app exposes GET `/health` under its compiled base path. It validates
  web configuration and confirms only web-process responsiveness, not backend
  availability, schema readiness or an authenticated product journey.
- Startup owns explicit child handles/groups from creation and requires the
  selected web process's ready signal plus HTTP health, not an unrelated open
  port. Combined admission stays false until all selected components are ready.
  SIGINT/SIGTERM or component failure withdraws admission, drains owned tasks,
  signals only owned groups, and reaps child exits: 60s startup,20s graceful,
  further5s forced cleanup. Linux invocation-local subreaping adopts orphaned
  descendants; non-reaping owned-child checks prevent signaling a reused group
  number after ownership ends. There is no global process-table scan or registry.
  Cleanup failure is failure, never rollback evidence.
- Backend images do not require Node/web artifacts. Web deployments do not
  receive backend credentials. Combined and separate backend topology uses one
  external PostgreSQL authority; no shared SQLite volume among replicas.
- One fixed-name prepare Job has native Helm pre-install/pre-upgrade weight-10
  and Argo PreSync wave-10 annotations with before-creation/success cleanup,
  `backoffLimit:0`, finite deadline, restart Never and no service-account token.
  It references already existing DB/key Secrets directly, not later ConfigMaps,
  ServiceAccounts or same-app Sync Secrets. Failed Job evidence is retained.
- Backend Services remain private ClusterIP. Any optional external gRPC edge
  requires explicit HTTPS/HTTP2 proxy/TLS configuration and distinct Admin
  routing; plaintext internal hops must be documented. Public web origins are
  HTTPS except actual loopback-bound CLI HTTP. Image paths must match their
  compiled prefixes; do not invent runtime prefix rewriting.

## Ownership and Integration

| Workstream | Owner | Owned paths | Depends on | Output | Focused validation |
| --- | --- | --- | --- | --- | --- |
| Foreground ownership and health | /root | rust/apps/chapchoo; root Cargo manifests/lock; both apps src/app/health/route.ts | reviewed backend/web configuration | optional backend admission, owned web children, process health, signal/failure drain | Rust CLI/unit tests, app quality/build |
| Linux packaging and independent images | /root/accounts-rpc-adapter | scripts/package-linux.py; scripts/tests/test_package_linux.py; images/chapchoo/** | shared archive/CLI contract and standalone outputs | verified matching Node archive, safe traced asset assembly, independent Docker targets | safe assembly/extraction/unit tests; no actual acceptance delegated |
| Helm/Argo packaging | /root/accounts-protocol-generation | infra/charts/chapchoo/**; scripts/check-chart.py; scripts/tests/test_chart_contracts.py; testenv/chapchoo/chart-values.yaml; testenv/chapchoo/argocd-example.yaml | shared CLI/env/health/image contract | chart, single preparation hook, examples and chart guide | helm lint/template, bounded chart contract tests; no cluster operations |
| Native architecture CI and actual acceptance | /root | .github/workflows; .dockerignore; testenv/chapchoo except assigned files; E2E package; docs; remaining integration tooling | all returned implementation | native x86/ARM artifact runs, same3 product journeys, image/chart and approved live evidence | root executes all integrated acceptance and reviews exact-head CI |

- Workers read applicable instructions, do not stage/commit or modify shared
  manifests/lock, and return focused evidence. Root alone changes shared
  interfaces, executes integrated acceptance, handles findings and opens the PR.
- Integration order: fixed layout/env/health contract → independent bounded
  workstreams → root composition/artifact assembly → native/package/image/chart
  checks → same3 product journeys → frozen same-reviewer review → corrections,
  hooks, PR and exact-head CI. Do not delegate actual acceptance execution.
- Single independent read-only reviewer: `/root/accounts-reviewer`, unchanged.
  Root requests the whole phase review only after integrated validation is stable.
  High-risk corrections get targeted same-reviewer re-review under current policy.

## Validation, Removal and External Prerequisites

- Root checks include frozen frontend install/quality/tests/builds/stories,
  Rust format/Clippy/tests, real SQLite/PostgreSQL and generated RPC gates,
  source-free archive assembly/native libraries, both Linux architecture startup,
  static assets, process/readiness/signal/partial-failure/foreign-resource safety,
  and explicit web-child environment canaries. Verify no target host Node/pnpm/
  Turbo dependency, no orphaned owned child, and no automatic command replay.
- Prefer native architecture CI runners for x86-64 and ARM64 verification;
  do not install privileged global binfmt handlers as an implicit prerequisite.
  Record actual tested libc/native/runtime prerequisites. A cross-build or
  emulation-only result is not an unqualified native-run claim.
- Reuse the existing three account/team journeys for installed package and real
  deployment acceptance. Keep role/transport/failure matrices at lower layers.
  Missing prerequisites fail or remain explicitly blocked; do not skip to green.
- Chart validation separately checks one dual-controller Job, preexisting-secret
  admission, failure retention/no token mount, private backend ports, separate
  routing, read-only readiness and >=30s orchestration grace. Chart rendering is
  not a deployed health/TLS/product result. Image rollback is not DB rollback.
- Live writes require exact cluster, namespace, ingress/TLS/exposure, image
  availability, database/data retention and existing fixed-key Secret scope plus
  Argo full-application sync authority. None is supplied yet. Secret values are
  not requested in chat or read by denied Secret tools. Finish all independent
  implementation before batching required participant decisions.
- Remove only occupied layout placeholders. Keep account/data authority and the
  fixed original key. No new legacy/protocol compatibility layer is needed.
  Verify no Hello World/duplicate theme reinstatement or broad unrelated cleanup.
- Promote reachable Specs and normalize their code-path discovery as needed.
  Mark Requirements/Design implemented and remove all feature plans only after
  complete package and approved live acceptance. If live scope remains blocked,
  preserve plans/snapshot state and report the unachieved gate honestly.
- Context: PR #5 at fd9b68a is open/CLEAN, independently reviewed with its single
  recovery finding corrected. Exact-head CI40937884806 passed browser3m36,
  Rust4m4 and validate6m57. Earlier PRs remain open; no agent merge or cluster
  write has occurred. Source web/PQ acceptance is not packaged/deployed evidence.

## Root integration checkpoint

- REQ-5 correction: reuse the existing Team translation in all four locales for
  collaborative badges; remove the unused Workspace key and add regression
  coverage without changing layout/theme/personal terminology.
- Fixed actual image discovery: web build requires the real RPC generator and
  Rust/vendored-protoc prerequisite. Retain both toolchains only in the build
  stage; no generated-code bypass or runtime toolchain addition.
- Actual local x86-64 package: pinned Node24.22.1 and latest ownership-hardened
  Bookworm release assembled successfully. Extracted SQLite preparation,
  both web/static/font resources, SIGINT and closed listener checks passed from
  an unrelated temporary CWD with no host tools on product PATH. The three real
  PostgreSQL journeys and focused HttpOnly-cookie ordering test also passed;
  fixture cleanup exited successfully.
- Actual independent images: backend/general/Admin built and ran nonroot with
  read-only roots and writable temporary locations; private RPC dispatch and
  graceful stop passed. Each web image omits the opposite app; backend omits Node.
- Root assembly14 and chart10 tests passed, including33 negative render cases.
  Full Rust gate discovered the explicit dependency-boundary list needed the
  approved Unix ownership dependency `nix`; update that exact list and rerun,
  retaining the exact-equality dependency guard.
- Native CI now asserts actual x86-64/ARM64 runner/Docker architecture, builds
  Bookworm release and matching Next output, verifies Node/ELF/archive checks,
  executes lifecycle/browser/image acceptance and uploads only verified archives.
  Existing source checks remain. Configuration alone is not native ARM success.
- Root final gates passed: full Rust format/Clippy/workspace tests and exact
  dependency boundaries; Node/Rust live RPC71 checks on each of SQLite/PostgreSQL;
  frontend format/lint/typecheck and all UI/core/general/Admin/E2E unit tests;
  production and both Storybook builds; all-files pre-commit; docs18 tests/catalog;
  actionlint for both workflows. Local Storybook requires the existing explicit
  SWC native cache environment; the first run omitted it and failed, then the
  correctly configured rerun passed without changing dependency or product code.
- Independent review, PR/exact-head native CI and separately approved live
  Helm/Argo/TLS acceptance remain pending. No implementation marker or plan cleanup
  is authorized by local artifact success alone.

## Independent review corrections

The same assigned reviewer read the complete integrated53-file tree2f76b65 and
identified two P2 correctness findings, with no other grounded findings:

- Continue supervising the first-ready web child's exit while the other starts,
  then recheck both children immediately before admission. Add an explicitly
  synchronized regression in both General-first and Admin-first orders.
- Reject duplicate ingress-nginx root locations between enabled web HTTP and
  gRPC catch-all routes on the same class/host. Keep distinct web prefixes,
  disabled routes and different classes valid; add negative and positive renders.

Root applies both corrections and reruns Rust/RPC, the latest Bookworm bundle's
lifecycle/browser acceptance, independent images and chart contracts before
same-reviewer targeted re-review. These restore approved readiness/routing
contracts; Design delta remains None.

Root's first correction recreated the still-pending readiness future after the
other child's completion. Actual package startup failed; retaining that future
also preserves its already-consumed Ready receiver. A second synchronized
regression starts both health probes before releasing either, then completes
General followed by Admin. The first-ready-exit test separately observes prompt
failure detection and the approved20s+5s cleanup; its initial5s whole-process-exit
assertion incorrectly conflated those budgets. It still requires no final ready
announcement, failure exit and disappearance of both owned groups.

## Requested Home snapshot follow-up

The requester subsequently selected the Azents snapshot + Home Argo deployment
pattern and `chapchoo.internal.hardtack.me`. This refines the environment of the
existing deployment work; it does not add product/versioning authority.

- Keep PR #6 atdea1927 and its exact-head source/native successes. Implement this
  focused follow-up on `feat/snapshot-publishing` stacked on `feat/accounts-delivery`.
- Root owns native snapshot workflow/publication metadata and prefix-image checks;
  root also owns the separate Home worktree's dedicated namespace/PostgreSQL/PVC,
  ExternalSecret references, private Gateway root/Admin-prefix routes and updater.
- Publish three independent multiarchitecture image indexes only after each
  native runner executes the images. Home pins chart SHA and images from one
  authenticated complete source run, never caller-supplied dispatch image data.
- Home prerequisites remain a separate Application from product PreSync hooks.
  Initial acknowledgement/auto-sync stays off until actual prerequisites and
  first immutable image publication are confirmed. The updater preserves this
  operator state; missing values never become guessed image digests or keys.
- Current main/PRs are not merged by the agent. GitHub Actions Secret/variable
  management returned403 with this installation, and no Secret values are read
  or copied. Record required Infisical/GitHub references and batch activation
  approval after completing review, both PRs and their exact-head CI.
- Same reviewer `/root/accounts-reviewer` reviews the integrated follow-up across
  both repositories read-only; root owns all actual validation and corrections.
- Design delta: None for the initial snapshot follow-up; actual live
  installation/upgrade/TLS/product gates remain pending rather than being replaced
  by render or workflow-unit evidence.

## Approved secret feedback — revision37

The subsequent explicit implementation request approves the consolidated
PASSWORD-only, setup-proof generation and SECRET_KEY naming changes. Requirements
clarification, ADR-D38 and Design37/M44 record this before code changes. This is
an approved delta from revision36's broad chart-no-secret-creation boundary,
limited to first-admin proof. No existing DB or cryptographic format changes.

Root owns the Rust/config/client-fixture rename, separate proof hook/script/RBAC,
backend-only pinned kubectl, Home template/refs and documentation. The same
read-only reviewer checks the final cross-repository diff. Root validation covers
old-binary-created DB binding with the same key under the new configuration name,
negative previous-name admission, both DB RPC gates, actual images/native client,
archive browser journeys, idempotent/invalid/racing proof fixtures, chart hook and
permission ordering, actual URL escaping with reserved/space/Unicode characters,
all hooks and exact-head CI. No new actual cluster Secret access is performed.
