---
title: "Server Tools Revision 2 Authority and Feasibility Audit"
created: 2026-11-12
document_role: supporting
document_type: supporting-audit
tags: [tools, architecture, verification]
---

# Server Tools Revision 2 Authority and Feasibility Audit

The requester selected the uniform-row presentation and explicitly refined its
icons, primary-color rail, sender-label removal and normalized body previews.
This audit incorporates that authority and the component-render evidence.
After these checks and the visual refinements, the requester explicitly requested
implementation and PR delivery. Matching revision-2 approval is now recorded in
the Design; this audit itself is not the source of that authority.

Target: [tools-261112/DESIGN revision 2](tools-261112-server-provided-tools.md).
Requirements: [tools-261112/REQ](../requirements/tools-261112-server-provided-tools.md).
Decisions: [tools-261112/ADR](../adr/tools-261112-server-provided-tools.md), D1–D14.

Reviewed Design SHA-256:
`f95109abfa1949289e0243541847f183893100c739ed521578889ded07cd8885`.
Mechanism set: M1–M20, exactly the twenty rows in the target authority section.
Requester approval is recorded in the linked Design; implementation verification
remains incomplete and is not established by this audit.

## Scope and Method

The main design agent reviewed Requirements-to-mechanism coverage, reverse
mechanism authority, source ownership, replacement/removal scope and a credible
implementation/verification path. No delegated decision owner or independent
reviewer was used. Structural checks supplement, not replace, this semantic review.

Revision 2 preserves M1–M20. Its semantic delta is REQ-13/D14's explicit rendered
selection, M12's detailed presentation and corresponding replacement/verification
coverage. The unchanged architecture, limits, persistence, transport and rollout
analysis is retained from revision 1. Its historical reviewed hash was
`8356b22f24b3620d24063023bdef1c94adf6b5149f464533d71d2fa5fd171394`;
the current hash above, not that earlier draft, identifies this approval candidate.

Current repository evidence uses baseline
`83a41877867f3602f741da0a9de819fa52f76929`. The
[current-system research note](../notes/server-tools-261112-current-system-research.md)
records exact source seams and pinned external reference provenance. No product
code, schema or deployed process was retained as a product change. No authenticated
MCP, model, database or actual-product browser integration was run. Static
component renders used a temporary local Storybook fixture and are distinguished
from those unexecuted product acceptance gates.

## Authority Result: Pass at Design Level

All thirteen Requirements have an explicit coverage row and mechanism mapping.
All fourteen accepted decisions are retained; earlier accepted entries were not
rewritten to manufacture authority. New outcomes were recorded before synthesis.

| Mechanisms | Reverse authority finding |
| --- | --- |
| M1–M3 | Selected local HTTP relay, component RPC and native-owner capability with independent current server authority; no central MCP, Control tunneling or readiness permission grant |
| M4–M6 | Selected process-owned waiting/fairness and response stream; request-local cancellation conveys the chosen pre-dispatch arbitration without another durable handle or query platform |
| M7 | Existing harness authorship and capture/server copying retained; no response-before-history circular barrier |
| M8–M11 | Confirmed independent chat, server user receipt, stable server receipt order and preservation of retained original inputs; no native resend or history reconstruction |
| M12–M14 | Confirmed item mapping, subsequent selected uniform-row treatment and existing private read/deletion scope; separate chat reads do not erase native records or create an orphan archive |
| M15–M17 | Required finite budgets, existing cleanup ownership and preserving application; disclosed resource parameters add no configurable operating mode or performance promise |
| M18–M19 | Confirmed reconstruction exclusion and default-output/non-fallback policy, without work completion, waiting or continuation state |
| M20 | Required minimal E2E plus lower-layer coverage; no claim that unexecuted new tests have passed |

The internal cancellation signal is restricted to the current invocation, not an
arbitrary operation identifier. It changes no caller target or permission. It is
necessary to preserve explicit MCP cancellation separately from broken transport,
while retaining the selected single invocation lifetime. Local frame spelling and
helpers are not a new domain API or resend procedure.

The numeric bounds are explicit revision-1 parameters under the already required
finite mechanism. They are not represented as previously confirmed Requirement
values. They require exact encoding, race and capacity validation. Whole-Design
approval must include these visible bounds; changing material behavior afterward
requires another Design revision/approval.

### Removal and preservation review

The removal table identifies replacement authority for raw-native main-chat
publication, general tool reads used solely for chat, acceptance without chat
receipt, obsolete tests and stale Spec claims. It explicitly retains native
capture, source identity, supported user input, steering, Stop and responders.
Consumed-input cleanup remains; its lifetime no longer owns accepted chat.
Native source replacement does not become chat deletion. Existing authorized
owner deletion still cleans private conversation children.

## Findings Resolved Before Submission

1. Requirements retained old wording that treated final presentation and separate
   chat management as pending. Their normative sections now point to the confirmed
   REQ-13/D14 mapping and independently managed accepted chat.
2. A generic dropped-future signal would be insufficient evidence of explicit
   MCP cancellation. The Design now names the SDK's public typed cancellation
   notification hook and keeps HTTP/RPC loss distinct from that event.
3. The new tool payload budget must not silently reduce existing user-input size.
   The Design explicitly retains the existing 16-MiB user-input contract and
   bounded message-body continuation reads.
4. Historical transfer order cannot be reconstructed from unrecorded old commit
   sequencing. The Design establishes a deterministic migrated prefix before
   new admissions without inventing historical receipt facts.

No new business behavior or authority ID was added to resolve these findings.
The revision-1 findings remain resolved in revision 2. Neither revision is
implemented. Revision 2's subsequent implementation/PR instruction is recorded
separately as requester approval.

### Revision-2 presentation authority and evidence

- The earlier broad assent was not used as detailed visual authority. The later
  explicit option-1 choice and refinements now supply that authority under REQ-13
  and D14's appended clarification, without changing D1–D13.
- User input retains the user icon; only Agent publication uses the plane.
  Native/model text uses a text icon and an output label without the Agent name.
  Message rows have no visible sender label/background highlight.
- The rail resolves the current theme primary color, including Agent scope.
  Actual computed colors changed from the default to a different scoped palette;
  neither user nor Agent message rails remained fixed yellow.
- Collapsed publication actually rendered the normalized message-body beginning,
  using the existing preview function with a review-only presentation projection.
  Raw publication input/result were retained in the detail fixture.
- Desktop/mobile renders checked closed/open states, SVG paths, visible labels,
  body previews, theme linkage and horizontal overflow. Real disclosures opened
  from the collapsed state. The original screenshots were visually inspected.
- The fixture's temporary generic-icon substitution and title-free presentation
  copy are not proposed production classification rules. Production must use
  actual normalized publication identity; ordinary `other` tools are not messages.
  New service/native normalization and authenticated integration remain G1/G4.

## Source Feasibility Result: Conditional, No Identified Design Blocker

Existing source provides completed database operations, private scope checks,
component streaming, bounded input/capture reads, retained native ownership,
exact restoration and finite cleanup. Current disclosure/input/output components
and static component renders provide the selected presentation seams. New services and storage still have
to be implemented and tested.

Additional public SDK inspection used the source revision referenced by the
pinned native, `modelcontextprotocol/rust-sdk` commit
`98a54789b80110ad210495c4ad845c89e1980471`:

- `ServerHandler::on_cancelled` is a public typed notification hook.
- Streamable HTTP's public `SessionManager` exposes creation and closure, allowing
  bounded owner/session integration without private APIs.

This demonstrates a credible public API shape. It does not select a Git
implementation dependency, waive normal dependency policy, prove a published
release's behavior or establish actual authenticated compatibility. The selected
implementation dependency must provide the required public APIs and profile;
otherwise revisit feasibility rather than hand-code a substitute protocol.

### Required implementation acceptance gates

| Gate | What remains unverified | Required outcome |
| --- | --- | --- |
| G1 Actual relay/native combination | Effective 2025-06-18 negotiation, headers/scope, catalog collision and new/exact-resumed tool use | Model-free actual SDK round trip plus separately authorized actual-native evidence; unsupported combinations fail, never silently downgrade |
| G2 Database/application boundary | Atomic input/chat receipt, original migration prefix, old admission writer fencing and downgrade refusal on SQLite/PostgreSQL | Preserving migration and concurrency fixtures; no accepted input without chat and no native replay |
| G3 Queue/cancel/cleanup | Exact encoded limits, cancellation/dispatch races, fairness, lost receivers and started capacity | Deterministic tests with explicit synchronization/clocks and honest uncertainty after response loss |
| G4 Product/UI behavior | Message-only SSR chat, full body retrieval, stable pages, actual normalized publication identity, retained output and publication details | One minimal browser journey plus focused closed/open, preview, role-icon, sender-label and scoped-primary checks; no fabricated record, generic-tool misclassification or duplicate published bubble |

These are implementation acceptance conditions with concrete supported paths,
not evidence of successful execution. A discovered need for a new mode, state,
authority or recovery contract is a blocker to resolve through Requirements/ADR,
not a risk that can be ignored.

## Verification Inventory

- Structural audit: 13 numbered Requirements, 14 accepted decision entries,
  20 unique mechanism rows, complete requirement coverage and required Design
  sections. Approval metadata matches revision 2 and the exact M1–M20 set.
- Documentation metadata/snapshot validation and its 18 unit tests are the
  executable lightweight checks for these document changes.
- Relative file links, whitespace and exclusion of private workspace URLs are
  checked separately for all changed draft documents.
- Earlier research recorded 33 existing model-free Rust tests; they were not
  newly rerun as evidence of this unimplemented tool service.
- New evidence is limited to static real-component renders and their deterministic
  DOM checks. No actual model call, new native MCP acceptance, migration execution
  or implemented-product screenshot is claimed.

## Approval Boundary

The requester approved revision 2 with exact authority set M1–M20, its explicit
resource bounds, data-preserving application/rollback boundary, no-replay/no-
reconstruction policy and the acknowledged implementation gates. This audit does
not substitute for the requester's separate implementation/PR instruction.
Live model calls, operating-data migration, merge and deployment remain outside
that instruction.
