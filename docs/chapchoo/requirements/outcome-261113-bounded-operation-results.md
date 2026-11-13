---
title: "Bounded Operation Results Requirements"
created: 2026-11-13
document_role: primary
document_type: requirements
snapshot_id: outcome-261113
tags: [reliability, frontend, backend, agent]
---

# Bounded Operation Results Requirements

- Snapshot: `outcome-261113`
- Document reference: `outcome-261113/REQ`

## Problem

General and Admin repeatedly expose ambiguous operation results, conflate known
failures with missing observations, and leave users in states without a bounded
next transition. The required conversation-send capability is also named
differently from its approved contract and depends on tool discovery.

## Primary System Outcome

Every user-visible operation has a coherent path from admission to its confirmed
result or an actionable result-verification failure. Normal observation latency
does not appear as an unknown-result warning, and lost observations never grant
permission to duplicate a potentially completed effect.

## Supporting Scenarios or Effects

- A private Agent can intentionally send a message through its always-visible
  `send_message` capability.
- Chat input, deletion, interactive answers, cancellation and native preparation
  recover or end their observation budget without indefinite ambiguous UI.
- AI authentication, saved execution choices, Agent creation, Computer lifecycle,
  Team creation and account/Admin settings use the same outcome principles.

## Goals

- Eliminate avoidable uncertainty by completing state transitions.
- Bound unavoidable verification waits and provide useful recovery after timeout.
- Preserve accepted writes, exact identities, authorization and physical safety.
- Deliver independently reviewable, behavior-complete PRs.

## Non-Goals

- No automatic replay of potentially performed irreversible effects.
- No fabricated successful delivery, native retirement or resource absence.
- No unrelated layout redesign, service provisioning, merge or deployment.
- No deletion of diagnostic evidence merely to hide a product problem.

## Requirements

### REQ-1. Stable, directly available conversation sender

The production tool is named `send_message`, as previously approved. The native
model receives this essential capability without needing a search first. Other
integration tools retain their existing discovery policy. Fixed recipient,
authorization and intentional publication semantics remain intact.

**Acceptance criteria**

- Catalog, dispatch, presentation and executable fixtures agree on `send_message`.
- Actual native launch supplies the sender as an initial direct tool; discovery
  is not a prerequisite for calling it.
- No compatibility alias silently preserves the erroneous production name.

### REQ-2. Known outcomes stay known

Definitive rejection, not started, not applicable, historical metadata absence,
read failure and in-flight outcome verification are distinct product facts.
Acknowledged effects are not downgraded when a subsequent read fails.

**Acceptance criteria**

- Authentication completion survives failure to refresh its detail view.
- Mail delivery not configured is reported as known unavailability, not uncertain
  mail transmission.
- Confirmed observations replace obsolete local uncertainty.

### REQ-3. Bounded next transition for real result verification

Every real verification wait has an owner, expected observations and a finite
budget. Re-reading, reopening or reconnecting does not create an endless wait.
Before budget expiry, use quiet progress or no warning. After expiry, expose a
verification failure with its impact and a safe recovery path.

**Acceptance criteria**

- Device authentication uncertainty cannot escape both automatic status reads
  and time-bounded resolution.
- Chat preparation and Computer lifecycle cannot retain a normal-progress label
  indefinitely after their verification budget ends.
- Late authoritative results can supersede the timeout presentation.
- Timing out a waiter does not cancel or pretend to roll back a started effect.

### REQ-4. Complete General and Admin scope

Apply REQ-2 and REQ-3 to all audited production paths: retained/local chat input,
input deletion, interactive answer, stop, setup, predecessor retirement, supply
save/applied metadata, Agent authentication selection, connection mutations and
Device acquisition/cancellation, Agent creation, lifecycle commands and progress,
power/storage observations, Team creation, account/server display settings,
email and account/Admin error fallback.

**Acceptance criteria**

- Each path has a recorded disposition, verification and no unowned permanent
  unknown-result state.
- Read-only diagnostics retain evidence while normal surfaces express capability,
  impact and next action rather than raw uncertain enums.
- Shared behavior covers General and Admin and all supported localizations.

### REQ-5. Safe recovery and bounded client waiting

Recovery checks the original operation identity where an effect may have begun.
Known-safe rejections can be corrected and retried; uncertain physical effects
are not resubmitted as fresh requests. Transport and product observation budgets
must not leave browser controls permanently waiting.

**Acceptance criteria**

- Exact-once acceptance and existing idempotent request identity are preserved.
- Missing observations after a deadline do not become false nonexecution proof.
- Recovery remains available after a verification failure without granting new
  effect authority.

### REQ-6. Reviewable delivery and verification

Split the corrections into meaningful behavior-complete PRs with owned tests and
current-system Spec updates. Run integrated verification and independent review;
report each PR and current-head CI honestly.

## Fixed Constraints

- Preserve existing user data in SQLite and PostgreSQL.
- Preserve conversation/native source identity and recipient boundaries.
- No live message-send tests, restarts or deployment without explicit permission.
- Required E2E stays limited to core product flows; combinations belong in lower
  layer deterministic tests.

## Open Assumptions

- Existing finite budgets are starting evidence, not authorization to invent
  irreversible actions or new operational modes after timeout.
- Any newly discovered choice that changes authority or recovery beyond these
  outcomes is reported separately while independent corrections continue.

## Confirmation

The requester directed all previously summarized feedback and all audited gaps to
be fixed on 2026-11-13 KST, then explicitly required meaningful PR boundaries.
This records that implementation instruction and the already stated two-part
state-transition policy; it does not claim approval of unpresented mechanisms.
