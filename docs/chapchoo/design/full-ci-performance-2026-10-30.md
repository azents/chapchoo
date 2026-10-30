---
title: "Full CI Performance Investigation"
created: 2026-10-30
tags: [ci, performance, developer-tooling]
document_role: supporting
document_type: supporting-verification
---

# Full CI Performance Investigation

## Outcome and constraints

Full validation must be fast even without changed-path selection. Target:
pre-commit and Rust verification in seconds, image builds within 3–5 minutes.
Preserve both native architectures, Rust/SQLite/PostgreSQL, generated RPC,
frontend quality/unit builds, static Storybooks, assembled browser journeys,
archive/checksum and independently running native images. No publishing,
deployment, real credential access or merge is authorized by this work. The
requester explicitly added replacing SES with SMTP, separate build outputs reused
by Docker, correct build caches, and a Mac Docker-development Linux wrapper.

## Baseline evidence

Commit `2669daf04ac46e906cfd047f240ecc11b48ac068` (same implementation now merged
as `2c345f1`), attempt 1:

- CI run [41021287606](https://github.com/azents/chapchoo/actions/runs/41021287606):
  Rust job 245 s, validate 367 s, browser 311 s. Workflow created 07:38:10 UTC;
  its final job completed 07:44:40 UTC (390 s including queue).
- Native run [41021287593](https://github.com/azents/chapchoo/actions/runs/41021287593):
  x86-64 job 780 s; ARM64 job 631 s. Final native completion 07:51:28 UTC.
  The slowest full-check completion across the two workflows is the native job,
  not the shorter CI workflow.
- Validate's repository hook step: 244 s. It executes the same complete Rust
  script as the separate Rust job and mutating frontend checks before explicit
  frontend quality steps. The duplication is visible in its actual hook logs.
- x86-64 native steps: host web build 99 s; Bookworm release builder 289 s;
  archive assembly/download/compression/extraction 80 s; later native image
  build/acceptance 230 s. Host and Docker web builds are separate compilations.
- Cargo's own release line reports 264 s of the 289 s Docker builder step.
  Final application compilation starts at 252 s. This is not a per-crate timing
  allocation: concurrent crate starts cannot establish individual durations.
- Rust logs separately report dev/check compilation 83 s and test compilation
  97 s. Compilation dominates much more than the individual test bodies.
- The lock contains 375 packages (not all necessarily compiled in one target).
  Default AWS config enables SSO/OIDC/STS; SES defaults include SigV4a and both
  HTTPS feature paths. Rustls selects aws-lc-sys native cryptography. Mail is a
  mandatory build dependency even when runtime mail configuration is absent.
  The subsequent explicit SMTP replacement removes that vendor credential-chain
  dependency rather than silently disabling features of an otherwise retained SDK.

Branch-protection required-check configuration could not be read with the App's
permission (403). Report all five observed PR checks rather than pretending their
required/optional policy was independently verified.

## Candidate ledger and implemented mechanisms

1. **Remove duplicate ownership:** Rust quality remains in the Rust job; frontend
   quality remains in explicit read-only steps. CI pre-commit omits exactly those
   duplicate invocations. Local commit hooks and generated RPC contracts remain.
2. **Retain build work safely:** toolchain/dependency-aware Cargo caches, cached
   pre-commit environments, and Turbo/Next compiler reuse. Per-job Cargo keys
   avoid simultaneous unrelated writers; tests still execute on every run.
3. **Build once, package separately:** reusable native web and pinned Bookworm
   release-binary build jobs export architecture-matched outputs. Same-run tar
   artifacts preserve symlinks and hidden Next files; both image and archive
   packaging consume them. The final Dockerfile has no Cargo/Next/pnpm build.
   Packaging layers have architecture-scoped cache keys, separate from web and
   Cargo compilation caches. Local Mac/Linux scripts run target builds in Linux
   builder containers first, then package with the same named-input contract.
   Mac dependencies never enter Linux artifacts; Apple Silicon defaults ARM64.
4. **Fast standard gzip:** level 1 instead of GzipFile's default level 9, retaining
   standard tar.gz, deterministic epoch and input validation. Local mechanism
   sample on 67,223,771 bytes of already-built standalone JS/JSON/native inputs:
   level 9 took 2.282 s / 12,239,751 output bytes; level 1 took 0.287 s /
   15,381,830 bytes. Exact decompressed input equality passed. This is one local
   mechanism sample, not a claimed whole-CI speedup; upload overhead must count.
5. **Compilation visibility:** preserve Cargo HTML timing for clippy/test and
   native release builds so subsequent cold-compilation work targets measured
   crates rather than inferred start order. Timing collection does not alter
   production build profiles.
6. **Replace SES with SMTP:** the explicit request removes AWS config, SES,
   SSO/OIDC/STS and aws-lc from the resolved graph (375 → 316 packages), using
   Lettre with Ring/WebPKI TLS. Optional mail, manual links, safe errors and
   no-automatic-resend remain. See the
   [SMTP verification](smtp-transition-verification-2026-10-30.md) for settings
   and actual local SMTP/TLS/auth tests.

## Coverage mapping

- Rust formatting, Clippy, all Rust tests and dependency boundaries: retained in
  the Rust job and unchanged local hook. Only the redundant CI hook call is gone.
- Frontend format/lint/typecheck: retained as explicit read-only checks; local
  hook behavior unchanged. Unit suites, all three Storybooks and product builds
  are retained.
- Generated RPC output and live Node/Rust interoperability: retained in CI hooks.
- Native release/ABI and independent image boundaries: same digest-pinned
  Bookworm compiler and image targets, now one shared build graph.
- Native archive ELF, pinned Node checksum, safe files/symlinks, deterministic
  archive/checksum, SQLite lifecycle and PostgreSQL product journeys: retained on
  x86-64 and ARM64 using exported builder inputs.
- Local structure/support tests assert target/export ownership and retained
  acceptance obligations. They do not substitute for native execution evidence.

## Verification state

Before the final CI-only debug-symbol adjustment, `367e61e` passed both complete
workflows twice on the unchanged SHA (CI run 41030029813 and native run
41030029885, attempts 1 and 2). First successful step times: repository hooks 36 s,
Rust gate 65 s (8.63 s dev/check + 25.07 s test compilation, plus test bodies),
web build commands 12–15 s, release compilation 31–36 s, packaging 90–96 s.
These runs reused compilation caches primed during earlier failed packaging
attempts; they are not cold-start measurements. Initial failed packaging logs
showed remote Git-context interpretation of local artifact paths; `source: .`
corrected it. Explicit filesystem reads document the intended artifact inputs,
not stronger isolation than the pinned Bake action's existing automatic policy.

The final candidate disables debug symbols only for CI Rust dev/test profiles
to reduce the remaining codegen/link cost. Test assertions/behavior and local
profiles remain unchanged; release optimization remains unchanged. Cargo cache
keys include the profile environment, so this candidate must prime a distinct
compiler-output cache and be measured independently.

- CI structure, artifact safety/orchestration, package and Snapshot contract
  tests: 31 passed. The orchestration fixture exercises Mac arm64 command flow;
  it is not a claim of execution on a physical Mac.
- SMTP TLS/auth, failure/no resend, downgrade refusal and app configuration checks
  pass, as does the chart gate. Full integrated Rust/RPC and hooks are in progress.
- Independent integrated read-only review found no material findings.
- Actual local Linux builders and experiment CI are in progress; no images are
  published. Commit hooks and final exact-head CI remain delivery prerequisites.
- Performance acceptance is **not established**. Preserve cold-cache priming
  separately; obtain at least two complete successful same-SHA full executions,
  recording every job/step, cache restore/save overhead, runner-time cost and
  cold/warm status. Do not claim the requested time budget from this draft.

## Structural follow-up and Runtime interruption

The requester additionally required removing unused scaffolds/placeholders,
shortening redundant internal directory prefixes, locating executable E2E under
`typescript/apps/e2e`, and placing real reusable fixture/process prerequisites
under `testenv/chapchoo`. npm/Cargo identities and product behavior remain unchanged.
Turbo rejects workspaces outside its root, so workspace/lock/Turbo authority now
lives at the repository root. Next traces that root, and bundled app entries are
`web/{general,admin}/typescript/apps/{web,admin-web}/server.mjs`.

All nine CI checks passed on pre-structure `8d75dac`; this does not establish the
structural follow-up. Fresh structural Rust quality, SQLite/PostgreSQL live RPC
(71 checks each), frontend format/lint/typecheck, app/shared unit suites (49),
boundary checks (32), and documentation/build/package/Snapshot contracts (50)
passed locally. Exact reset/Admin-denial copy and 375px overflow checks moved to
app-owned executable component tests (web 2, Admin 1), which passed before their
removal from E2E. All three core journeys and browser cookie ordering remain.

Independent review discovered that raw TypeScript testenv exports depended on
Playwright transpilation. The corrected package emits JavaScript/declarations;
7 support tests and 5 plain-Node public-export smoke tests pass. Independent
re-review verified the correction without further material findings.

A Runtime restart interrupted assembled-browser and Linux Docker revalidation.
The requester identified ephemeral-storage exhaustion; temporary logs and Docker
state were lost while source changes persisted. Interrupted runs are not passes.
After restart Docker reported zero images, containers, volumes and build cache;
filesystem-wide free space was 59 GiB, which does not establish the previous Pod's
ephemeral-storage allowance or peak use. Do not repeat heavy local Docker builds;
use existing native CI acceptance gates for the new structure. No global cache
prune, cluster mutation, publication, deployment or merge was performed.

## Build-only pipeline and single E2E correction

`7faccd6` passed all nine checks (CI 41038659849 and native 41038659888,
attempt 1). The requester then corrected the validation design: build pipelines
must only build, and existing E2E must not be duplicated. Attempt 2 of that older
configuration was canceled rather than measured as the final candidate.

The native workflow is now a reusable build-only pipeline called by CI. It builds
both architectures' images/archives and uploads outputs without browser, PostgreSQL
or product-runtime acceptance steps. The existing single `browser-e2e` job depends
on those builds, downloads the same run's x86-64 archive, verifies its checksum,
and runs the three core journeys from that bundle. Source-mode CI E2E and the two
per-architecture product E2E repetitions are replaced by this one gate; no extra
verification workflow is added. ARM64 remains native build/ELF/assembly evidence,
not a second product-test execution. Snapshot publication-specific image acceptance
is unchanged. Validate also drops its duplicate aggregate product build: native
web-build owns those builds; library prerequisites and Storybook/component checks
remain. Runner scope/order changed, so earlier durations are historical and
must not be labeled final-candidate performance. Exact new-head CI is pending.
