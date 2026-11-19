---
title: "Conversation Work phase 1: live ending evidence"
created: 2026-11-18
tags: [engine]
---

# Phase execution plan

- Branch/base: `feat/stride-conversation-work-native` → `a787ee2089aa9a277f41e5adf8e0de58bda0ee03`.
- PR boundary: native-owned evidence for completed root work; no automatic turns.
- Inputs: [approved revision 17](../design/stride-261118-conversation-work-continuation.md), actual Codex/ACP ending and Resume observations.
- Approved mechanisms: `M3`, `M9`; `REQ-2`, `REQ-5`, `REQ-8`, `ADR-D1`, `ADR-D8`, `ADR-D23`.
- Design delta: `None`.
- Non-goals: Control reporting, server authorization/admission, restoration transport, `send`, native input changes and UI.
- Interface: an actor-scoped read of the current local ending evidence (exact native source, prior work identity, captured occurrence); separate from foreground and retained capture APIs.
- Removal obligations: None; foreground and Stop retain their existing contracts. Tool removal belongs to phase 3.

| Workstream | Owner | Owned paths | Output | Validation |
| --- | --- | --- | --- | --- |
| Native evidence | Primary agent | `rust/crates/harness/src/runtime*`, `rust/crates/harness/src/lib.rs`, native-harness Spec | Positive terminal evidence minted only at live ingress, with an exact Stop fence | Harness tests including source, replay, abnormal ending, duplicate and later work boundaries |

- Integration order: live evidence issuer, actor access, fixture tests, Spec alignment, validation, independent review.
- Independent review: `/root/stride-reviewer`, read-only after integration.
- Final validation: pinned Rust format/test/clippy, documentation validator and diff checks. Released-runtime probe remains complementary.
- Scope drift: no reporting or admission effect; no replayable authority from ordinary snapshots or retained capture.
- Checkpoint: phase started; dependent server and restore integration not yet implemented.
