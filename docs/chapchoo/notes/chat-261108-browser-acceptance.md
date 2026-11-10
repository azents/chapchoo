---
title: "Codex Conversation Deterministic Browser Acceptance"
created: 2026-11-10
tags: [conversation, testenv, frontend, verification]
---

# Deterministic Browser Acceptance

## Scope and environment

The Phase 5 integrated working tree passed the representative native conversation
journey using `pnpm --filter @chapchoo/e2e test:conversation`'s Playwright
configuration. The run used actual PostgreSQL, foreground backend/Control,
production web listeners, Docker Provider and execution-local Connector. The
existing-CLI selection explicitly chose the isolated deterministic app-server
executable. Official managed preparation supplied Node 24.22.2 and ACP adapter
2.6.0. The fixture reports compatible Codex CLI 0.165.0; it is not the real CLI.

The delegated Linux x86-64 Engine was Docker 29.8.2. The container retained PID 1,
UID:GID 10001, private PID/IPC, read-only root, dropped capabilities,
no-new-privileges and the unchanged native scope checks. The built development
binary emitted a weak GLIBC 2.39 warning in the Bookworm runtime. Successful
execution here does not certify release ABI or ARM64 compatibility.

No real credential or model request was used. Synthetic account authentication
and fixture login are separate from real Codex entitlement or model acceptance.
These results do not establish production deployment, merge readiness or remote
CI/review results.

## Observed journey

- Public prerequisite RPC created the private Agent and reached a connected
  actual Docker execution. The owner selected and saved the explicit existing
  CLI through the browser's Codex setup surface before first input.
- Browser text submission traversed ordinary owner upload/admission and native
  insertion. The official adapter delivered the first assistant utterance.
- The inline native form began without preselected consent. Explicit `stable`
  selection and the exact typed note reached the app-server fixture.
- The original assistant message updated under the same identity. A distinct
  second assistant utterance appeared without merging the two.
- Online activation reread the current source. Navigation away and reentry
  preserved both messages, original choices and the retained typed answer.
- Saving managed supply while the native session remained live changed the saved
  selection only. The separately observed actual-applied existing path remained
  unchanged through reentry; saving did not restart or reconfigure the process.
- Individual deletion completed and owned fixture cleanup succeeded. No daemon
  pruning, SQL state insertion, mutation retry or security-profile relaxation
  was used.

## Read observations

One successful run measured:

| Observation | Result | Interpretation |
| --- | --- | --- |
| Region reads before first input | 0 | SSR hydration did not duplicate the initial read before interaction |
| Input to first visible assistant | 58,419 ms | Includes new managed distribution preparation; not steady-state generation latency |
| Answer submission to visible update | 2,365 ms | Synthetic native output, ingestion, owner read and browser rendering combined |
| Last completed region response to visible update | 440 ms | Upper bound at the assertion, not a rendering microbenchmark |
| Online signal to reread response | 135 ms | Read-only reconnect check, not native-process recovery |
| Navigation reentry to visible retained content | 796 ms | Server entry plus visible assertions |
| Region HTTP reads across the journey | 58 | Includes preparation observation; not a count of underlying gRPC/SQL work |

These are bounded single-run observations, not performance targets, load capacity
claims or repeated same-SHA benchmarks. Native/model acceptance and the final
integrated diff still require their own evidence.

## Corrected integration failures

The deterministic app-server originally omitted the native user item's
`clientId`; the official adapter therefore could not acknowledge insertion.
It now returns the exact `clientUserMessageId`. The adapter's title-generation
helper also receives its own ephemeral thread rather than reusing the primary
conversation identity.

The browser originally put bigint ordinals into JSON-hashed query keys. Once
known-record hints became nonempty, a client TypeError prevented further fetches
although direct owner reads succeeded. Read-hint and paging input values now use
canonical int64 decimal strings, with server validation and exact bigint
restoration. The owned boundary tests cover large values and invalid encodings;
the successful native browser journey verifies the integrated correction.

A failed earlier deletion left one restarting owned execution after Connector
credential withdrawal. That run remained failed; after its owning processes had
exited, its exact reserved container and volume were independently verified and
removed. Later successful cleanup is not retroactive success for that failure.
The fixture image is explicitly supplied to this isolated lane and was inspected
without an ownership label in Phase 5. Phase 6 corrected general image-label
handling while retaining exact reserved ownership labels and all profile checks.
The [integrated acceptance report](chat-261108-implementation-acceptance.md)
records the failed regression and real metadata-bearing lifecycle verification.

## Metadata-bearing image rerun

After that correction, the same representative browser journey and individual
cleanup passed on the metadata-bearing Linux amd64 image recorded in the
integrated acceptance report. Initial region reads remained zero. The bounded
observations were input-to-first assistant 53,928 ms (including new preparation),
answer-to-update 2,360 ms, completed-response-to-visible upper measurement 368 ms,
online reread 248 ms, reentry 832 ms and 54 region reads. Saved-versus-applied
supply remained distinct without restarting the live native session.

This is another synthetic app-server run, not a real Codex/model call, platform
certification or repeated performance benchmark.

## Post-correction current-code rerun

The complete representative journey and individual cleanup passed again using
the code at `8ab27a43ce667583367abf1f2a95c95c3be3c63c` and a freshly compiled
development executable in image
`sha256:e93b0caa30dd94d5972b60a67d977bc51a84d91c424f3636fe838742d8675031`.
The image deliberately carries the standard OCI title and exact source-revision
labels. The narrowed restricted-profile rule admits those descriptive labels,
not operational or arbitrary OCI-prefixed extensions.

This run includes the corrected authentication/startup and source-qualified
observation wire, incoming tool field storage, owner recovery, finite-body extent
reads and initial-view UI assembly.
It still uses the explicit synthetic app-server through the official adapter;
no real credential or model request was introduced.

| Observation | Result |
| --- | --- |
| Initial region reads | 0 |
| Input to first visible assistant, including preparation | 63,797 ms |
| Answer to visible update | 2,356 ms |
| Completed response to visible update upper measurement | 376 ms |
| Online reread | 203 ms |
| Navigation reentry | 654 ms |
| Region HTTP reads | 63 |

These remain one-run plumbing observations, not a model performance benchmark.
Normal individual cleanup passed and an independent exact-image query found no
remaining containers. An earlier attempt was blocked by the explicit Docker
transport-environment guard before resource creation; another was interrupted
by a Runtime restart during web build, before the fixture's Docker Provider
started. Neither attempt is counted as passed or retroactively certified by this
successful fresh-fixture run. The restart cause was not established.
The [large-body forced-append correction](chat-261108-implementation-acceptance.md#deterministic-large-body-read-correction)
is verified separately by twenty forced interleavings on each real database and
the actual consumer. This short representative journey alone is not evidence of
large-body scheduling behavior.

## References

- [Private conversation web](../spec/domain/conversation-web.md)
- [Verification boundaries](../spec/domain/acceptance.md)
- [Executable browser lane](../../../typescript/apps/e2e/README.md)
- [Approved Design](../design/chat-261108-text-conversation.md)
