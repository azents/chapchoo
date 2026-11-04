---
title: "Provider Authentication Implementation Evidence"
created: 2026-11-04
tags: [provider, authentication, verification]
document_role: supporting
document_type: supporting-validation
---

# Provider Authentication Implementation Evidence

This report concerns [provider-261104](provider-261104-authentication.md), approved
Design revision 1 / M1-M14. It is local disposable-environment evidence, not a
production deployment, public image publication or end-to-end TLS claim.

## Environment and Commands

- Linux x86-64, pinned Rust 1.99.1, Node 24.22.1, pnpm 11.26.0, Helm 4.2.4.
- Required PostgreSQL suites used newly initialized private Unix-socket servers,
  not an operator database. SQLite suites used temporary files.
- Runtime-owned Docker daemon supported disposable kind clusters and synthetic
  Computer containers. No production Kubernetes context or secret was used.
- `CARGO_INCREMENTAL=0 bash scripts/check-rust.sh`: PASS; workspace Clippy with
  denied warnings, rustdoc, policy/dependency boundaries and 226 passing tests
  across 58 reported suites; zero ignored. Connector policy remains independent.
- `CARGO_INCREMENTAL=0 bash scripts/check-rpc.sh`: PASS; generated clients checked,
  Node/Rust live RPC 67 checks each on SQLite and PostgreSQL.
- `python scripts/check-chart.py`: PASS; 19 offline chart contracts and strict lint.
- `python -m unittest scripts.tests.test_setup_proof`: PASS, four tests.
- `pnpm --filter @chapchoo/testenv typecheck`, `test`, `lint`: PASS; 18 pure support
  tests plus five built-export tests. No UI count or scope was expanded.
- `python -m unittest scripts.tests.test_ci_structure scripts.tests.test_build_artifacts scripts.tests.test_validate_docs`:
  PASS, 30 tests. Document metadata and local-link checks passed.
- Changed-file pre-commit passed. Rust/RPC/chart hooks were not rerun inside that
  invocation because their authoritative commands were already run separately.

## Actual Kubernetes Authentication and Lifecycle

`python scripts/check-provider-auth-cluster.py --image <current-run-local-image>`
creates, owns and deletes one disposable kind Kubernetes 1.35 cluster. It loads a
current built backend/Connector image, creates isolated PostgreSQL/root prerequisites
without printing secrets, and installs the actual chart with only Kubernetes
Provider enablement beyond the common installation inputs.

Observed PASS:

1. Chart creates separate execution namespace and Control/Provider ServiceAccounts.
2. Real Provider projected SA token authenticates through actual TokenReview;
   the Provider connects and canonical SYSTEM/KUBERNETES identity is stored.
3. Explicit TokenReview-backed exchange returns 900-second operational authority;
   exact request recovery returns the original token.
4. Wrong audience, unpermitted Provider identity and network Trust requests reject.
5. Public account/Team/Agent APIs create a Computer, its real Pod/PVC/Connector
   connect, and stop/start/delete complete without replaying lifecycle mutations.
6. The test's owned cluster is removed even on an ordinary failed run.

The same acceptance step is connected to the existing native integration CI job,
using that run's native binary/image artifacts. It is not substituted with mocked
TokenReview or Pod Running status.

A first exploratory Kubernetes 1.37 run authenticated the Provider but exposed an
existing short Service-name endpoint problem: execution Pods are in another
namespace, so Connector DNS could not reach Control. The chart now supplies a
namespace-qualified Service address, with an offline assertion; the full lifecycle
then passed. Failed exploration was not counted as successful acceptance.

## Actual Standalone Docker

A temporary SQLite/root/setup fixture launched the real command
`serve --standalone-docker` against the explicitly selected disposable Docker
socket/current built image. No Provider auth file or external owner token was
supplied. The generated acceptance client verified public Trust rejection and
completed actual Agent/Computer/Connector create, stop, start and delete through
the in-process Provider Trust path. Owned server and temporary data were cleaned.
This is Linux local-daemon evidence, not Mac, remote Docker or network Trust proof.

## Failures and Corrections

- Rust 1.99.1 incremental compilation panicked in an unchanged diesel-async
  AsyncFunc trait query. Disabling incremental compilation allowed the same
  source/toolchain suites to complete; no product fallback was added.
- New fixture failures from missing deferred owner membership, display defaults
  and personal-Team deletion constraints were corrected in the new fixture, not
  by relaxing product schema. Both-engine scoped registration tests pass.
- Initial lint failures in new test helpers were fixed. Unrelated lockfile
  resolution drift was removed; the lock diff contains required dependency edges,
  not an opportunistic package upgrade.
- A first Standalone readiness harness watched the second buffered log line and
  timed out despite ready startup; the harness used the existing owned backend
  readiness event and the actual lifecycle passed. This was not a product fix.

## Independent Review and Scope

The single read-only reviewer `/root/provider-auth-reviewer` reviewed the stable
integrated implementation against Requirements, ADR and M1-M14 and reported no
material findings. The primary agent owns code changes and execution evidence.

Existing Provider shared-owner bearer configuration/RPC and generic chart alias
were removed. SYSTEM/USER/TEAM scoped random-token authentication is backend
capability only; no registration UI or new onboarding workflow is claimed.
Team deletion revokes TEAM Provider authority while preserving referenced cleanup
identity. Provider migration invalidates old keys/routes but preserves canonical
Computer/native/Connector data. Docker Helm deployment is not implemented.

No production merge, credential invalidation, deployment, restart or data deletion
was executed. The dependent private deployment PR needs a compatible canonical
main Snapshot before activation; current published old binaries do not implement
these new authentication inputs.
