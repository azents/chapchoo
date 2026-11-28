---
title: "Resumable Chat Execution Preparation Decisions"
created: 2026-11-28
document_role: primary
document_type: adr
snapshot_id: chat-261128
tags: [chat, execution, reliability]
---

# Resumable Chat Execution Preparation Decisions

Authority: [chat-261128/REQ](../requirements/chat-261128-resumable-execution-preparation.md).
Observed baseline: `5b56100`; the fix branch also includes unrelated repository maintenance through `b82a6cd`.

## Fixed boundaries

- Retain the exact conversation and native source. Do not add a priming turn, invent an idle or completion event, or replace the saved context.
- Leave input acceptance, the durable forwarding grant, reporting, and non-replay policy unchanged. Native preparation and possible input delivery are different facts.
- Retain existing physical retirement and active request/tool fences, and use unmodified released harnesses. No timer, disconnected link, or historical record proves that work finished.
- Keep current integration ownership, approvals, outbound policy and failure isolation. No production operation is authorized by this snapshot.

## Material decision map

- `chat-261128/ADR-D1`: preserve eager preparation on connection. Accepted by the requester on 2026-11-28 (KST). Addresses REQ-1 and REQ-2.
- `chat-261128/ADR-D2`: how an unresolved **preparation** settles visibly without revoking possible effects or opening replacement authority. Pending; discuss after D1. Addresses REQ-2 and REQ-3.

Reporting a known preparation failure as such through the existing typed result is fixed. Its projection cannot change the delivery contract. The choice in D2 concerns unresolved cases for which the preparation owner has no terminal proof. Helper names, test fixture details, and exact placement of equivalent checks are not decisions.

### D1. Choose when to prepare and attach a cold native owner

Status: the requester accepted preserving eager startup on 2026-11-28 (KST). The approval does not establish retirement evidence for a changed selection or decide D2.

The connection reconciler can report a successful exact Resume. After a user submits an input with saved external settings, the forwarding transaction still waits for configuration selected for that input. The cold observer starts with an incomplete work inventory, and the refresh gate requires a complete and quiescent current owner. Retained history is not that evidence. An isolated browser test on the baseline seeds history, saves an optional server, physically stops the Computer, explicitly starts a successor, and verifies exact retained Resume and `Ready` before sending once. The new preparation stays possible and the accepted input has no forwarding evidence after 45 seconds. The fixture uses a deterministic Codex protocol peer and an unreachable optional server; it does not assert real tool invocation.

- **Preserve eager startup with one owner-local preparation lifecycle (recommended).** Take the applicable saved configuration at the actual launch boundary, before opening the exact native session. Separate a new physical owner's evidence from the retained source and make later selection changes use the same lifecycle. Configuration already applied to the current owner should not force its replacement merely because an input has arrived, but per-input and route-bound selection authority must still be proven. For changed configuration on a live owner, require complete owner-local work evidence and confirmed retirement before replacement. This keeps the existing ready-on-connect experience and applies to configuration saved after resume as well as before it. Feasibility depends on exact released producer evidence; a fresh process, Resume response, or thread `Idle` snapshot alone is not sufficient.
- **Load retained history on connection but defer cold native attachment until an input is eligible.** This prevents starting a cold owner with an obsolete selection and may avoid unused startup work. It delays native capabilities/readiness until work arrives, changes existing connection-startup behavior, and still requires a safe warm-owner refresh when settings change after the session is running. This would need explicit approval of the changed startup behavior.

Do not adopt an MCP-specific first-message bypass. It skips selection and leaves the equivalent deadlock for other configuration after restore. The released MCP reload endpoint is not a general replacement for applying the combined prompt, authentication and launch-local tool selection, and adoption of this per-thread declaration scheme through that endpoint has not been demonstrated.

Feasibility gate for the recommended approach: the Codex 0.171.0 client Resume path emits an idle lifecycle after its response, and the native goal extension can use that lifecycle to continue an active goal. The later design must identify the supported producer/ingress evidence and ordering that account for such work, approvals, children and tool effects; otherwise it must report an unresolved preparation rather than imply safe retirement. No native goal behavior is changed by this proposal.

### D2. Separate preparation outcome from selection and replacement authority

Status: pending. Known cleanup failure, unstarted wait, lost report and a physical owner that may still exist are separate. A direct compound cleanup error can currently leave the operation in `POSSIBLE` because the Setup result classifier has no matching failure branch. Correcting that known-failure projection does not settle a lost report or grant a successor. For genuinely unresolved work, compare explicit unresolved feedback while retaining the operation with ending a demonstrably unstarted preparation under its exact owner. Do not choose either via a client timeout.
