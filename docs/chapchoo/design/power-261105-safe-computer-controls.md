---
title: "Safe Computer Power Controls Design"
created: 2026-11-05
implemented: 2026-11-05
tags: [computer, lifecycle, frontend]
document_role: primary
document_type: design
snapshot_id: power-261105
---

# Safe Computer Power Controls Design

## Authority and current gap

[Requirements](../requirements/power-261105-safe-computer-controls.md) define the
three requested owner outcomes. [ADR D1](../adr/power-261105-safe-computer-controls.md)
selects persistent desired state without a separate queue. Current code already
opens a controlled Stop/Restart dialog, but its event-to-mutation boundary needs
explicit regression verification; the production missing-dialog report is not
yet reproduced. All pending work currently blocks Start. Browser DTOs omit the
already-existing wire action and expose a generic uncertainty label.

## Design Authority

- Design revision: `1`

| ID | Mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Explicit Stop/Restart consent bound to the captured predecessor | REQ-1 and current computer-web consent contract | required |
| M2 | Persistent Running target update against the still-current Stop; retain its physical identity and receipts | REQ-2, ADR-D1 | decided |
| M3 | Old-execution Running exclusion, retirement-before-successor and original-storage gates | REQ-2, ADR-D1, current lifecycle/storage contracts | derived |
| M4 | Operation-specific status, accepted next target and useful uncertainty explanation without fabricated power | REQ-3 and current truthful web projection | required |

## Data, interface and transition boundaries

No database migration, additional queue, receipt table or protocol change. The
existing desired state is the only persisted future target. The existing lifecycle
request transport carries Start and an expected predecessor. Under existing
owner/Team/Provider/Computer locks, exact original-request replay/fingerprint and
predecessor checks occur before the target-update branch.

Start against the exact current Stop retains the original request/hash/action and
native authority. If desired is already Running, return current progress without
another effect. Otherwise require existing management/capability/original saved
storage admission and update only the target while retirement is active. If Stop
already completed, admit the ordinary Start under the same locks. Existing
retirement transactions map Running target to Starting and preserve exact old
identity. Late Running reports for that old Stop execution are observation-only
at RPC and rejected as lifecycle completion at persistence. The distinct successor
can still confirm Running.

The public summary derives the effective Start action once the old execution has
retired, so delayed-start and UI copy describe successor work rather than stale
Stop provenance. Stored Stop action remains available for idempotent desired-state
recovery against that predecessor. A later lifecycle/Delete changes the predecessor
and rejects stale target writes. Delete remains terminal. No new cancellation UI
is added.

Expose the already-existing wire action in the browser DTO. Stop with pending
retirement shows shutdown progress; when its target is Running show that startup
will follow shutdown. Primary Start follows backend eligibility even while current
power is Running or Unknown; accepting intent never fabricates stopped power.
The accepted Running target prevents duplicate fresh starts. Normal stop/start,
stale/error locks, preparation retry and destructive controls retain their owners.

## Confirmation and copy

Keep the controlled confirmation boundary and captured request identity. Verify
click/touch/keyboard selection alone does not submit; cancellation and polling do
not change consent. Do not claim a root-cause correction without reproduction.

Operation-aware labels replace generic uncertainty for General and read-only Admin.
Keep physical power and connection facts separate. Explain unverified completion
without promising that polling will necessarily recover lost evidence. Localized
Korean, English, Japanese and French keys remain aligned and are reviewed in context.

## Test Strategy

One existing assembled Computer E2E remains the primary product journey: real
Control/Provider/Connector, PostgreSQL, web and RPC over the supported Kubernetes
SDK fixture. Extend its confirmed Stop/Start path with a deterministic shutdown
barrier, requesting Start during that interval, asserting no successor before
release and original saved-storage reuse. No live cluster credentials are needed.

Narrow tests own exhaustive combinations: SQLite/PostgreSQL target acceptance,
exact replay/predecessor/owner checks, retirement-first race, uncertain old Running
rejection, distinct successor, deletion supersession and storage gates. Docker
native correlation remains the original Stop scope; validate unchanged submission
counts in its existing integration fixture. Domain tests cover eligibility and
summary progress. Component/hook checks cover confirmation, cancellation, responsive
copy, stale locks and operation presentation. Missing test prerequisites fail;
synthetic fixtures never certify live Kubernetes or runtime recovery.

## Feasibility and risks

Repository research confirms the existing retirement/successor gate is reusable
and that overwriting the active request would invalidate native work. This design
preserves it. A target setter cannot promise a new receipt for an unstored Start
UUID; the original receipt and exact predecessor remain authoritative. Existing
lost terminal evidence may still prevent retirement indefinitely. This feature
neither fixes nor hides that limitation. No live mutation, deployment or merge.

## Removal and Replacement

| Existing behavior | Authority | Replacement | Boundary | Verification |
| --- | --- | --- | --- | --- |
| Blanket pending-operation Start rejection for the exact stopping predecessor | REQ-2, ADR-D1 | Target-only Start admission | Domain and persistence | Both-DB lifecycle tests |
| Generic unconfirmed result label | REQ-3 | Operation and consequence copy | General/Admin presentation | Localized unit/component checks |
| DTO drops existing wire operation | REQ-3 | Typed operation projection | web-core | DTO validation tests |

No saved state, native resources, original receipts or uncertain-effect fences are
removed. Confirmation is retained and verified, not replaced by a new control mode.

## Design Approval

- Mode: Collaborative
- Decision owner: requester
- Approved on: 2026-11-05
- Approved Design revision: 1
- Approved authority IDs: M1, M2, M3, M4
- Approved scope: direct implementation of the three requested outcomes with the
  explicitly selected persistent desired-state model and no separate queued Start.
  Derived identity/evidence gates preserve existing safety rather than add a new
  choice. Any broader recovery, cancellation or authority change requires renewed
  requester direction.
