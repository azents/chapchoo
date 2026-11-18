---
title: "Conversation Work implementation plan"
created: 2026-11-18
tags: [backend, engine, frontend]
---

# Conversation Work implementation plan

[Requirements](../requirements/stride-261118-conversation-work-continuation.md) ·
[ADR](../adr/stride-261118-conversation-work-continuation.md) ·
[Design](../design/stride-261118-conversation-work-continuation.md), revision 17,
approved `M1`–`M10`. Design delta: `None`.

The primary agent owns implementation and integrated validation. The single
independent reviewer is `/root/stride-reviewer`; the primary asks for read-only
review of each stable phase diff.

1. **Live ending evidence (M3, M9).** Capture and qualify current root endings
   before foreground clears the work target. Prove exclusion of restored,
   stopped and failed work. No product effects. See
   [phase plan](stride-261118-phase-1-native.md).
2. **Goal and follow-up authority (M1–M5, M9–M10).** Build trusted restore and
   report paths, atomic admission and reset, and independently fenced internal
   input delivery. Prove authority with isolated native fixtures, faulted
   delivery tests and supported database tests. Verify the release-supported
   native paths, including the 8-KiB full goal projection.
3. **`send`, tool cutover and summary (M6–M9).** Implement scoped tool
   transitions, wait and Chat projection. Retire `send_message` at discovery,
   dispatch and instructions together. Test effective New and exact Resume
   exposure and run the core Conversation Work browser journey. Use reviewed
   UI behavior without importing unrelated working-tree changes.

Stack the phases in dependency order; capture the validated commit, review and
remaining integration boundaries at each checkpoint. Product authority and
exact feature effects are not proven by deterministic loopback models alone.
Verify Docker prerequisites before acceptance, exercise supported SQLite and
PostgreSQL contracts, and do not present a skipped credentialed lane as passing.
Release only verified native combinations. Merge and deployment require
explicit approval. Promote current Specs and remove plans after integrated
implementation and validation.
