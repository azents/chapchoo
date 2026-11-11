---
title: "Chat Input Handoff and Computer Status Decisions"
created: 2026-11-11
document_role: primary
document_type: adr
snapshot_id: chat-261111
tags: [chat, lifecycle, web, computer]
---

# Chat Input Handoff and Computer Status Decisions

- Snapshot: `chat-261111`
- Requirements: [chat-261111/REQ](../requirements/chat-261111-input-handoff.md)

## D1. End the input record at confirmed native acceptance

**Status:** Accepted requester outcome, 2026-11-11.

For REQ-2/REQ-3, the original pending input is completely removed when the execution-local native owner reports `Inserted` or `Steered`. The existing typed result is the smallest current evidence of actual Agent acceptance. Upload completion, dispatch enqueue and missing results are not sufficient. Model completion and captured-record correlation are not prerequisites.

Do not retain a delivered-input archive, a deleted-input tombstone, or a replacement admission sequence/capability. The requester explicitly rejected the proposed per-Agent admission counter and instructed removal. This intentionally ends indefinite receipt recovery for a consumed request. Missing completed input is not known non-delivery and does not grant automatic resend. The service cannot remember an arbitrary completed owner request forever without retaining identity; do not pretend otherwise or smuggle in the rejected mechanism.

Failed/uncertain work remains governed by its actual unresolved lifetime, not relabeled as successful consumption.

## D2. Separate native session facts from pending input lifetime

**Status:** Derived from REQ-2/REQ-3 and unchanged source-preservation constraints; feasibility work in progress.

Removing an input does not erase the actual native conversation or grant fresh replacement context. Input-owned content/acceptance/delivery facts must end, while independently required session/source authority remains authoritative. No per-input archive may be reintroduced to preserve it. Final mechanism must be tested against setup rejection, unresolved work, source replacement and retained-source requirements.

## D3. Show owner state instead of a catch-all unknown label

**Status:** Accepted requester outcome, 2026-11-11.

For REQ-4/REQ-5, known lifecycle work remains the primary state. Native replacement is presented as restart. During unavailable internal observation, preserve the last-known same-source execution state and append only a spinner. Display retention does not restore current effect authority or enable Send/Stop. Confirmed failures remain failures.

Do not invent a last-known state on initial absence, or infer native failure from a read timeout. The previous checking-status caption and separate replacement caption are removed.

## D2 feasibility evidence

Both database engines verify input consumption followed by another explicit
submission and exact retained-source preparation with `fresh_native_allowed=false`.
The existing native binding and independent startup history supply this authority;
input absence supplies none. The pending-row count remains only a current
admission condition, not a replacement completed-request counter. Failed,
unresolved and differently bound historical preparation is preserved. This
resolves D2's implementation feasibility obligation without changing its decision.
