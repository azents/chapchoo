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
without an ownership label. Image-label handling in general provider acceptance
remains a separate Phase 6 finding; this lane does not weaken production image or
resource ownership checks.

## References

- [Private conversation web](../spec/domain/conversation-web.md)
- [Verification boundaries](../spec/domain/acceptance.md)
- [Executable browser lane](../../../typescript/apps/e2e/README.md)
- [Approved Design](../design/chat-261108-text-conversation.md)
