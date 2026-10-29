---
title: "Local PostgreSQL and PGlite Evaluation"
created: 2026-10-29
tags: [database, rust, feasibility, testing]
document_role: supporting
document_type: supporting-review
snapshot_id: accounts-261029
---

# Local PostgreSQL and PGlite Evaluation

## Status and question

This report records the investigation of local PostgreSQL/PGlite alternatives.
After that investigation, the requester selected SQLite/PostgreSQL with Diesel
under ADR-D28. Neither local PostgreSQL bundling nor PGlite is a deliverable for
this cycle. Measurements below remain evidence, not current selection authority.

## Measured environments

- Linux x86-64 Runtime; rustc 1.99.1, SQLx 0.9.0.
- Node 24.22.1; official `@electric-sql/pglite` 0.5.13 and
  `@electric-sql/pglite-socket` 0.2.16, with exact package lock.
- PGlite reported PostgreSQL 18.6 compiled to wasm32/Emscripten.
- Native comparison: PostgreSQL 17.11 in a private temporary cluster, normal
  durability settings. The same Rust probe operations were used on both.
- PGlite used file-backed storage and the official socket server with eight
  allowed connections. The tested SQLx pool held four connections. A separate
  restricted variant used one connection and nonpersistent statements.
- All servers/data were isolated research fixtures and cleaned up. No deployed
  or product database was modified. PostgreSQL major versions differ; these
  results do not establish exhaustive version-independent behavior.

## Functional results

| Case | PGlite + official socket | Native PostgreSQL baseline |
| --- | --- | --- |
| SQLx connection and extended-protocol bound parameters | Passed in single-connection flow | Passed |
| Typed native ENUM reads/writes and invalid role rejection | Passed | Passed |
| Deferred composite owner-membership FK | Passed creation, ownerless-commit rejection, owner-removal rejection | Passed same cases |
| Ownership transfer with former-owner role update | Passed | Passed |
| Dropping an idle transaction after insert | Subsequent read confirmed rollback | Same |
| Disconnect with an uncommitted transaction | No row remained when another connection read | Same |
| File data across normal shutdown/restart | Committed marker survived | Same |
| Four retained SQLx connections with default prepared statements | Failed with SQLSTATE 42P05: prepared statement sqlx_s_1 already exists | Passed |
| Same pool with statement_cache_capacity(0) | Still failed with 42P05 | Passed |
| Four connections with per-query persistent(false) | Sequential bound requests worked; concurrent SELECT 11 / SELECT 12 returned 11 for the second result; reproduced in a fresh run | Expected 11 and 12 |
| One connection with persistent(false) | Repeated bound requests and two concurrent caller requests serialized correctly | Passed |
| Independent-row read while another connection holds an open transaction | Waited beyond the 800 ms observation bound | Returned other row before transaction ended |
| Connection-local application_name | A saw client_b after B changed its setting | A retained client_a |
| FOR UPDATE NOWAIT against a row locked by another connection | Waited beyond 800 ms instead of returning a lock error | SQLSTATE 55P03 immediately |
| statement_timeout=100ms then pg_sleep(1) | Query completed without the timeout error | Timeout error, connection remained usable |

A zero process exit for an observation case does not mean semantic parity: the
probe deliberately records differences such as settings leakage or ignored
NOWAIT/timeout. Default and zero-cache suites repeated those differences.
The nonpersistent concurrent wrong-result observation was reproduced twice;
this is a bounded finding for these exact versions, not a diagnosis of every
PGlite adapter or a claim about the core PostgreSQL SQL implementation.

The socket source explains two important boundaries: it serializes requests
through one PGlite engine and prefers the transaction-owning client while a
transaction is open; its CancelRequest handler explicitly ignores cancellation.
Multiple TCP clients are not equivalent to independent native PostgreSQL sessions.

## Cancellation protocol check

A separate PostgreSQL wire-protocol client obtained BackendKeyData, sent
`SELECT pg_sleep(2)`, and sent a CancelRequest on another connection after 250 ms.
The intentional delay establishes an elapsed-time cancellation test, not test
ordering for an unrelated operation.

- PGlite completed in approximately 2.001 seconds without an error.
- Native PostgreSQL returned SQLSTATE 57014 after approximately 0.252 seconds.
- Both accepted a subsequent query in this bounded check.

This confirms the documented cancellation difference at the wire boundary.
Client-side future cancellation cannot be assumed to cancel PGlite's work.

## Size and memory evidence

### Native binary distribution example

The third-party `theseus-rs/postgresql-binaries` release 18.6.1 contains real
PostgreSQL builds. It is a candidate distribution, not an adopted dependency.

| Platform-specific archive | Compressed download | Sum of unpacked regular-file sizes |
| --- | --- | --- |
| Linux x86-64 GNU | 38.6 MiB | 109.7 MiB |
| Apple Silicon | 84.7 MiB | 251.0 MiB |
| Intel Mac | 87.8 MiB | Not measured |
| Windows x64 ZIP | 46.5 MiB | Not measured |

Archive listings were read directly; unpacked sums exclude filesystem allocation
and symlink overhead. Only the target platform would be shipped. Retaining an
archive inside the app/cache adds its size after extraction. App/web runtime,
extensions not in the archive, user data, backups and future WAL/log growth are
additional costs.

A separate native PostgreSQL 17.11 Linux sample measured an empty initialized
cluster at about 38.4 MiB allocated on disk. Six idle PostgreSQL processes had
about 27.8 MiB summed PSS and 60.2 MiB summed RSS. Summing RSS double-counts shared
pages; configured shared_buffers=128MB is not the measured resident footprint.
These are a single idle sample, not memory reservations or workload upper bounds.

### PGlite installed package and process

- PGlite package tarball: 9,320,802 bytes, approximately 8.9 MiB compressed.
- Socket package tarball: 79,862 bytes, approximately 0.08 MiB compressed.
- npm metadata gives PGlite unpacked size 26,345,363 bytes and socket 356,897
  bytes. The actual installed dependency tree occupied about 28 MiB, excluding
  the already installed Node binary/runtime.
- Empty file-backed PGlite data occupied 40,013,824 bytes, approximately 38.2 MiB.
- Fresh initialization readiness took approximately 0.93-0.95 seconds in two
  runs; reopening existing data across the probes took approximately 0.13-0.21
  seconds. These are local observations, not a controlled startup benchmark.
- Node process RSS sampled immediately after fresh initialization was about
  677-698 MiB; existing-data startup samples were about 383-445 MiB. A separate
  sample after cancellation queries measured approximately 710 MiB PSS.

These samples include Node, WASM memory and initialization allocations, without
GC/memory tuning or steady-state workload characterization. They must not be
represented as unavoidable minimums or compared as a controlled performance
benchmark against the native idle sample. They do establish that a smaller
compressed package did not automatically produce a smaller resident process in
this tested setup. Claims such as a 3MB WASM asset do not describe the complete
npm tree, Node runtime, data directory, or process memory.

## Recommendation and remaining verification

PGlite is a real viable engine for local single-connection SQL execution and the
basic SQLx flow worked. It should not be dismissed as unable to accept SQLx.
However, **the tested official socket adapter is not a transparent replacement
for normal multi-connection PostgreSQL in this application**. Prepared-statement
collisions, concurrent wrong results, shared session settings, transaction-wide
serialization, and cancellation differences are material compatibility costs.

A local-only restricted mode with one connection and nonpersistent statements
is plausible and passed the narrow query test. It would require explicit serial
execution, connection/session-state discipline, timeout/cancellation handling,
and actual account workflows. Separately hosted general/Admin processes would
still need coordination; limiting each process to one connection is not a
single global connection. This is new operational behavior, not elimination of
all backend differences by using the PostgreSQL wire protocol.

Real local PostgreSQL remains the stronger recommendation if identical SQL,
locking, pool/session semantics and long-lived maintenance are the priority.
PGlite merits reconsideration if the smaller package is decisive and a deliberately
serialized local DB mode is acceptable. Neither choice is approved by this report.

Not tested: hard power loss/SIGKILL recovery, major upgrades, full migrated product
schema, SQLx migration advisory-lock behavior, extension parity, long-running
memory/CPU load, TLS/authentication suitability, macOS/Windows runtime behavior,
or PGlite SQLx query! offline-metadata generation. No universal safety or durability
claim is made from normal restart and basic rollback tests.

## Investigation notes

The first native-baseline restart harness omitted a log redirection for the
restarted server; inherited output pipes kept its capture waiting. The private
server was stopped, cleanup completed, and the harness was corrected to use a
restart log before rerunning both suites. This was a test-runner issue, not a
PGlite or PostgreSQL product failure. Results above come from the completed runs.

## References

- [PGlite description](https://pglite.dev/docs/about)
- [Official PGlite socket documentation](https://pglite.dev/docs/pglite-socket)
- [Socket implementation](https://github.com/electric-sql/pglite/tree/main/packages/pglite-socket)
- [Native binary release example](https://github.com/theseus-rs/postgresql-binaries/releases/tag/18.6.1)
- [PostgreSQL upgrade requirements](https://www.postgresql.org/docs/current/upgrading.html)
