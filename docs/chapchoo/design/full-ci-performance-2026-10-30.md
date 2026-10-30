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
