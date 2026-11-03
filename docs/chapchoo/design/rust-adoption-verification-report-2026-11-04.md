---
title: "Rust Semantic Convention Adoption Coverage and Verification"
created: 2026-11-04
tags: [rust, conventions, quality, refactoring, verification]
document_role: supporting
document_type: supporting-verification
---

# Rust Semantic Convention Adoption

This maintenance report supports [rust-261104/REQ](../requirements/rust-261104-engineering-conventions.md)
and the [research and enforcement report](rust-conventions-research-2026-11-04.md).
It records inspection coverage, disposition and verification boundaries. It is
not a new product Design, convention index or permission to merge or deploy.

## Inspection boundary

The initial main snapshot was `210db5354b11d580e48fdf74a1d68eb1563ab85c`:
all **105/105** tracked Rust source files were read and compared with the semantic
rules. The independently developed Docker snapshot was
`439c01e87210bbfad1395d13704a7b9dc7d38e79`: its **79/79** changed or new Rust
files were inspected, including the complete new native adapter and the changed
shared implementations. Git inventories matched the inspection ledger without
missing or extra paths. Docker's total source inventory was 141 files; the
79-file delta overlaps the main inventory and must not be added to 105 as if
all files were new.

| Owner | Main files | Docker changed/new files |
| --- | ---: | ---: |
| App composition and CLI | 13 | 13 |
| Crypto | 8 | 0 |
| Domain | 6 | 3 |
| Docker adapter | 0 | 20 |
| Kubernetes adapter | 3 | 2 |
| Mail | 1 | 0 |
| Persistence | 26 | 16 |
| Generated protocol boundary | 3 | 2 |
| RPC | 33 | 20 |
| Service | 12 | 3 |
| **Total inspected** | **105** | **79** |

Generated protocol output was not hand-edited. New completion helpers and
regressions introduced during adoption were reviewed as part of their focused
changes; counts above describe the sealed audit snapshots, not a continually
changing file count.

## Disposition and delivery

[PR #53](https://github.com/azents/chapchoo/pull/53) establishes 16 semantic rules,
path-scoped loading and executable mechanical policy. The research report
records every Python/TypeScript concern's adaptation, inheritance, tooling or
exclusion and links the primary Rust/Tokio references. Mechanical rules remain
in compiler, Rustfmt, Clippy, rustdoc and boundary/loading tests, not prose.

The main-based application series preserves product contracts while addressing:

- [#54](https://github.com/azents/chapchoo/pull/54): outcome/error-state meaning.
- [#55](https://github.com/azents/chapchoo/pull/55): foreground/Provider owned work
  and primary-plus-cleanup completion.
- [#56](https://github.com/azents/chapchoo/pull/56): retained private native causes
  with safe public formatting.
- [#57](https://github.com/azents/chapchoo/pull/57): Control/runtime drain ownership
  and independent failures.
- [#58](https://github.com/azents/chapchoo/pull/58): persistence fixture completion.
- [#59](https://github.com/azents/chapchoo/pull/59): RPC fixture task/completion
  ownership and deliberate cancellation.
- [#67](https://github.com/azents/chapchoo/pull/67): app/Control/Connector/Web CLI
  fixture completion, backing-directory lifetime and nested RPC diagnostics.

The separate Docker follow-up series addresses:

- [#60](https://github.com/azents/chapchoo/pull/60): volume/container create
  rejection retains original Bollard causes and safe classification.
- [#61](https://github.com/azents/chapchoo/pull/61): persistent transport/SSH
  completion, explicit process reaping and physical-lifetime capacity. Ordinary
  request/body failures remain owned by their SDK consumer, not duplicate sticky
  process incidents.
- [#62](https://github.com/azents/chapchoo/pull/62): Provider effect lifetime,
  cancelled/reentered drain, initialization cleanup and consumed-worker failures.
- [#63](https://github.com/azents/chapchoo/pull/63): Docker-specific persistence
  fixture primary/schema-drop failures.
- [#64](https://github.com/azents/chapchoo/pull/64): SDK HTTP fixture adapter drain,
  acceptor shutdown and every request join after normal exercise failure.
- [#65](https://github.com/azents/chapchoo/pull/65): native app/external Provider
  fixture dependency ordering, explicit worker/child completion and nested causes.
- [#66](https://github.com/azents/chapchoo/pull/66): native-retirement RPC fixture
  owned calls, exact intentional aborts, service drain and nested diagnostics.

Crypto/domain/mail/configuration, transaction ownership, persisted identity,
applied migration history, authority/source checks, SDK selection and both
SQL engines were inspected. They were not rewritten merely to add wrappers,
traits, builders or superficial syntax changes. Changes stay with grounded
ownership/error findings. Public wire/API, security boundaries, original-volume
retention, uncertain effects and existing request/product-stop budgets remain
unchanged. Lower-layer regressions do not add browser journeys.

## Combined verification

The original main-based and Docker-based series were independently reviewable.
After the requester merged Docker feature #49–52, all open adoption branches
were rebased onto the frozen current main
`2bcfd04afdb733db6452d9c5a7df067139f2c64b`. Rebase checks were conflict/diff
checks, not an extra
independent review or full local verification round.

A disposable local integration tree combined main app head
`de5ce4582486fe65afe55d8089d2b21dff95b3a6` and Docker RPC head
`afe10f1af7a62d20d8228f5b82cb516da2ee3e69`. Its resolved source tree was
`169896dfbb93f0982b6c42d8fdc21f48603f1ef3`. The resolution retained the Docker
Provider's additional initialization/drain and persistent-effect semantics,
registered its native journey, and kept the main app fixture scopes. Shared
completion and persistence/RPC/app fixture helpers were identical rather than
duplicated implementations.

On that tree the full canonical Rust gate passed: formatting, all-target
Clippy, executable policy probes, private/public rustdoc, **210 tests in 54
reported groups**, and dependency boundaries. Actual Node/Rust RPC checks passed
**67 checks on SQLite and 67 on disposable PostgreSQL**. This is combined-source
verification, not a GitHub merge, deployment or live Docker/Mac claim. Later
nested-diagnostic-only corrections have their own executable regression and
mandatory hook evidence; remote exact-head CI remains attached to each PR.

## Evidence limits and lifecycle

- Each implementation PR receives independent source review, required commit
  hooks, exact-head remote CI and automated-review response handling. Earlier
  successful heads are not substituted for a changed head's CI.
- One #63 actual Docker restart/readiness run failed and its unchanged-head
  failed-job rerun succeeded. That is retained as intermittent failure evidence,
  not a demonstrated causal repair. Actual Docker acceptance evidence belongs to
  the corresponding CI run, not the synthetic SDK byte fixtures.
- Explicit fixture scopes cover normal `Result` completion. They do not promise
  cleanup after assertion panic or infer physical cancellation from a dropped
  waiter. A recorded deadline can coexist with continued owned completion.
- Linux/local OpenSSH evidence does not imply Mac/Desktop or every remote
  TCP/TLS/SSH combination was tested.
- Requirements and the research report are preserved as development records.
  No invented material architecture decision or primary Design is added merely
  to mark this maintenance snapshot `implemented`; this supporting report does
  not change product authority. Current product contracts remain in Living Specs.
- Temporary planning notes are removed from the delivered branches. Audit/gate
  outputs and the resolved local integration patch are retained outside the
  repository for reproducibility. Merge and deployment require separate approval.
