---
title: "Messages Before Computer Readiness Decisions"
created: 2026-11-16
document_role: primary
document_type: adr
snapshot_id: messaging-261116
tags: [conversation, computer, messaging, architecture]
---

# Messages Before Computer Readiness Decisions

- Snapshot: `messaging-261116`
- Mode: Autonomous for remaining technical decisions within the confirmed feature. The requester delegated choosing the smaller structural change and completing implementation without further technical questions on 2026-11-16. `/root/deferred-input-decisions` is the dedicated technical decision owner; product scope remains requester-owned.
- Authority: [confirmed Requirements](../requirements/messaging-261116-deferred-computer-input.md).

## Current System Framing

The outgoing input row currently owns accepted original text and a process-consumer lease. New admission starts a bounded process-owned processor; exact receipt recovery does not dispatch it. Missing Connector readiness makes that processor return, and processor completion or orphan expiry leaves the input in Error. This is attempt-liveness tracking, not durable Computer-readiness waiting.

Native setup and text forwarding have independent sealed admission. A committed possible-forwarding fact never grants another native delivery attempt after a waiter loss or process restart. Successful authenticated delivery consumes the outgoing input into an ordinary Chat receipt. Failed attempts keep their existing explicit recovery boundary.

Computer lifecycle admission is already durable and reconciled outside its transaction. It is Cloud-only and qualifies the current owner, Team, Provider capabilities, original storage, management evidence and expected Computer request. The exact current request provides an existing supersession boundary for explicit Stop; its physical-operation identity must remain intact.

## Fixed and Derived Outcomes

- `messaging-261116/REQ-1` and `REQ-3`: accepted messages need a durable readiness-waiting lifetime distinct from a native attempt; multiple messages retain their original text and order. Readiness absence alone cannot become an attempt error.
- `messaging-261116/REQ-2`: canonical Computer kind controls automatic power behavior. Docker is not a synonym for Local. Existing lifecycle admission and Provider effects remain separate.
- `messaging-261116/REQ-4`: the ordinary composer and normal placeholder remain unchanged by internal waiting.
- `messaging-261116/REQ-5`: older backlog cannot undo a newer explicit Stop. A new submission or manual Start can resume work; Stop does not delete the waiting text.
- Existing private ownership, account-session authorization, Team admission, deletion, source authority and native no-replay fences remain. Browser reads do not dispatch effects.
- Original-storage and Provider safety gates are not removed to make automatic startup appear successful. External effects stay outside completed database transactions.

## Material Decision Map

| Topic                                    | State              | Decision owner                     | Consequence                                                                                 |
| ---------------------------------------- | ------------------ | ---------------------------------- | ------------------------------------------------------------------------------------------- |
| D1. Durable waiting-work authority       | Accepted: Option A | Requester                          | Extend the outgoing input lifecycle as the single accepted-message authority.               |
| D2. Automatic startup admission boundary | Accepted: Option B | Dedicated technical decision owner | Durable accepted-work processing admits Start later under an exact Stop-supersession fence. |

D1 and D2 are accepted. Their options are grounded in the separate existing input and lifecycle transactions. Neither permits a browser timer to own startup or a later worker to override an intervening Stop.

### Product and feasibility checks

No additional product behavior is selected by this record. Before a complete Design, validate account-session expiry/revocation, initial source binding versus actual replacement, multiple inputs during loading and FIFO waiting beside existing explicit Retry. Any conflict with confirmed Requirements returns to the requester rather than silently relaxing authority or changing recovery.

### Agent-owned local details

Identifiers, file placement, helper boundaries, equivalent local typed representations, scan pagination and fixture composition belong to implementation only while they introduce no new authority, persisted source of truth, runtime mode, configuration or user behavior.

## D1 Brief: Durable Waiting-Work Authority

Related requirements: `messaging-261116/REQ-1`, `REQ-3`, `REQ-5`.

### Option A: Extend the existing outgoing input authority

Keep accepted original text and waiting/processing progress under the existing outgoing input identity. Add an explicit distinction between safe pre-attempt waiting/ownership and already admitted native work. A bounded server owner discovers eligible waiting work independently of the original request lifetime.

- One authoritative pending message and body; existing queue reads and explicit failure recovery can remain centered on that identity.
- The existing attempt-liveness and expiry contracts need a preserving replacement that cannot reclaim attempted or uncertain native delivery.
- Migration must preserve retained Error and unresolved rows without inventing readiness-waiting authority for them.

### Option B: Introduce a separate durable submission authority

A durable submission command owns accepted waiting text and eligibility. It hands off to the existing outgoing attempt model when Computer readiness permits.

- Separates indefinite readiness waiting from the existing bounded attempt lifetime.
- Adds a durable handoff and two related identity/lifetime boundaries. Original text ownership, receipt recovery, ordering, failure reads and consumption must agree across them.
- Requires preserving migration and protocol/read-model decisions for that handoff; it does not inherently require an external broker.

### Recommendation

Prefer Option A. The current outgoing row already owns exact submitted text, request recovery, visible queue identity and successful consumption. Extending its lifecycle avoids a second acceptance authority while preserving the separate native forwarding fence.

### Decision

Accepted by the requester on 2026-11-16: **Option A**.

The existing outgoing input identity and original text remain the single accepted-message authority. Its lifecycle must distinguish Computer-readiness waiting and reclaimable pre-attempt processing ownership from an admitted native delivery attempt. Backend restart may resume only work proven not to have crossed the native forwarding boundary. Existing Error and unresolved native-delivery rows are preserved without promoting them to fresh waiting work. No separate durable submission authority is introduced.

## D2 Brief: Automatic Startup Admission Boundary

Related requirements: `messaging-261116/REQ-1`, `REQ-2`, `REQ-5`; accepted `messaging-261116/ADR-D1`.

### Question

When a newly accepted message permits waking a stopped Cloud Computer, should eligible Start intent be committed with message acceptance, or should a durable server-owned queue processor admit it afterward?

This concerns completed database admission, not a physical Provider call. Physical power changes remain outside either transaction and under existing lifecycle reconciliation.

### Shared Requirements and Safety

- Accepting the message must also retain its applicable wake authorization and current Stop-supersession context durably under that same completed acceptance operation. A process-only follow-up or browser timer cannot own the wake request.
- The outgoing input remains the single accepted-message authority under D1. Neither option introduces another pending-message store or external broker.
- Canonical Local Computers never enter automatic power admission.
- Before Start admission, revalidate account/private-owner/Team authority, exact current lifecycle context, Provider capability, current management evidence, original saved environment and existing execution/uncertainty gates.
- Missing startup eligibility is distinct from actual startup failure. Neither alternative may discard an accepted body or bypass safety gates merely to produce a Start request. Where eligibility is not yet established, the accepted work remains subject to its qualified durable waiting/recovery path.
- An intervening explicit Stop withdraws older messages' wake permission. Merely rediscovering backlog after restart cannot capture a fresh predecessor and manufacture renewed permission. A new submission can establish a new applicable wake context.
- Reobserving an already accepted lifecycle operation is receipt recovery, not a fresh physical dispatch.

### Option A: Eligible Start Intent in the Acceptance Transaction

For a currently eligible stopped Cloud Computer, one completed Store operation records the outgoing input and Start intent. The physical effect is reconciled afterward. When current safety evidence does not yet qualify Start, preserve the accepted work and its wake context for later server-owned evaluation.

- The eligible fast path returns after both message and Start intent are committed, with no crash gap between those writes.
- Input admission must adopt lifecycle's User, Team, Provider and Computer lock order and safety checks before its conversation/input writes. Calling the existing completed lifecycle operation from an input transaction is not a substitute for one correctly composed operation.
- Provider-scoped locking joins ordinary message admission even though readiness waiting still needs a durable server owner.
- The immediate path and later eligibility path must implement one coherent Start authority and supersession contract.

### Option B: Durable Queue-Owned Start Admission

Message admission commits the original input and qualified wake context. The server-owned queue processor subsequently revalidates and admits Start through a completed lifecycle operation. It then waits for readiness and processes the original queued inputs in order.

- All automatic startup follows one durable queue-owned path, independently of browser lifetime.
- Ordinary message acceptance does not need the lifecycle transaction's Provider lock scope. Lifecycle admission retains its own complete lock order and physical-effect boundary.
- The interval between acceptance and Start intent is deliberate durable waiting, not an unowned fire-and-forget task. Restart recovery must rediscover it; Start admission and wake settlement need exact request/predecessor recovery to prevent duplicate intent.
- Startup intent may appear after the acceptance response rather than in the same commit. No new user-facing composition mode or placeholder follows from that timing.

### Recommendation

Prefer Option B. D1 already requires a durable server-owned readiness queue, and the existing lifecycle service already owns complete, qualified Start admission. One queue-owned automatic-start path avoids adding Provider locking and an additional immediate-start path to every ordinary input acceptance operation. Its crash interval is safe only with the shared durable wake and Stop-supersession requirements above.

### Decision

Accepted by the dedicated technical decision owner on 2026-11-16 under the requester's scoped delegation: **Option B**.

Message acceptance durably records the original input and applicable wake context together. The server-owned queue later reuses completed lifecycle admission for Cloud-only Start. Exact request/predecessor recovery preserves an active Stop's physical identity and prevents older backlog from adopting a newer Stop as fresh wake authority. Physical effects remain under existing reconciliation. Missing eligibility preserves waiting; actual failures and authority withdrawal retain their existing recovery boundaries.

## References

- [Conversation storage](../spec/domain/conversation-storage.md) — current acceptance, source, consumer and native forwarding boundaries.
- [Computer lifecycle admission](../spec/domain/computer-lifecycle.md) — current expected-request fences and physical power safety.
- [Conversation web](../spec/domain/conversation-web.md) — current composer and effect/read separation.
- [`inputs.rs`](../../../rust/crates/persistence/src/conversations/inputs.rs)
- [`input_queue.rs`](../../../rust/crates/persistence/src/conversations/input_queue.rs)
- [`input_owner.rs`](../../../rust/crates/service/src/conversation/input_owner.rs)
- [`input_consumer.rs`](../../../rust/crates/service/src/conversation/input_consumer.rs)
- [`conversation_processor.rs`](../../../rust/crates/rpc/src/conversation_processor.rs)
- [`computers.rs`](../../../rust/crates/persistence/src/computers.rs)
- [Completed persistence operations](../../../.claude/conventions/rust/completed-persistence-operations.md)
