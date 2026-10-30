---
title: "Independent Design Reference Workspace Requirements"
created: 2026-10-30
implemented: 2026-10-30
tags: [design-reference, frontend, developer-tooling]
document_role: primary
document_type: requirements
snapshot_id: reference-261030
---

# Independent Design Reference Workspace Requirements

- Snapshot: `reference-261030`
- Document reference: `reference-261030/REQ`

## Problem

Chapchoo has production web presentation components and app-specific stories,
but no dedicated place to author and review general-purpose design references
across target platforms. The existing presentation was initially brought over
from Azents and is provisional, not an approved Chapchoo design system.

## Primary System Outcome

Contributors and agents can run an independent design reference workspace,
review static cross-platform design examples, and determine where each design
representation or reusable web component belongs without depending on a running
product application or backend.

## Supporting Scenarios or Effects

- Existing shared web components can participate in design exploration.
- Experimental and non-web representations have a home before or independently
  of production-web reuse.
- Future design work can start from scratch without treating existing styling as
  approved product identity.

## Goals

- Separate general design references from product functional testing and app UI QA.
- Make component ownership and selective promotion understandable to humans and agents.
- Provide runnable, verified scaffolding and discoverable authoring guidance.

## Non-Goals

- Select or redesign Chapchoo's visual identity in this change.
- Catalog the existing palette, typography, theme, or screens as an approved design system.
- Extract portable tokens or implement native components or platform generators.
- Adopt another design tool or UI library, deploy a reference service, or change
  product behavior.
- Replace existing app stories, functional tests, or product review workflows.

## Requirements

### REQ-1. Independent cross-platform design reference

Provide a dedicated space for general design references, separate from app QA.

**Acceptance criteria**

- Contributors can start and statically build the reference independently of the
  product apps and backend, using documented repository commands.
- Its initial examples explain the reference structure and ownership, rather
  than assembling a catalog of existing product screens.
- Examples use static data and may use small local interactions needed to explain
  a design; real login, business API calls, and credentials are unnecessary.
- A web-rendered mobile or native representation is identified as a design
  reference, not proof of executable native behavior.

### REQ-2. Clear ownership and selective promotion

Design review and production-web reuse remain separate decisions.

**Acceptance criteria**

- Existing formal shared web components are reused from their owning common UI
  package; changes to those components are made there.
- Experimental components without a formal shared equivalent can remain local
  to the reference workspace.
- Web-rendered non-web representations remain reference-local, including after
  their designs are approved.
- A reviewed component selected for actual shared web use belongs in common UI.
  References import that implementation when it expresses the same design role.
- When common UI has web-specific behavior or constraints, reference may retain
  a distinct, more general design representation. Its role and relationship to
  the web implementation are documented; promotion does not require deleting it.
- Common UI and product apps do not depend on the reference workspace; the
  reference does not depend on product app implementations.

### REQ-3. Explicit provisional-design status

The scaffolding must preserve freedom to design Chapchoo from scratch next.

**Acceptance criteria**

- Reference onboarding and Agent guidance explicitly identify the current
  Azents-derived presentation as provisional.
- Initial examples are labeled as scaffolding or provisional reuse, not approved
  Chapchoo design choices.
- No existing style value is promoted to approved design authority by this work.
- Production styling and behavior remain unchanged by this setup.

### REQ-4. Discoverable documentation and Agent guidance

Contributors and agents can follow the intended workflow from repository entry points.

**Acceptance criteria**

- Documentation explains how to run, author, review, and selectively promote
  references, including the distinction between design approval and web reuse.
- Repository and relevant package Agent instructions direct future work to the
  correct owner and describe the reference's scope and provisional-design status.
- Guidance distinguishes reference-local experiments, shared production-web
  components, non-web representations, and app-specific stories.

### REQ-5. Verified scaffolding and boundaries

The new workspace participates in the repository's normal quality process.

**Acceptance criteria**

- Formatting, linting, type checking, and static Storybook building include the
  new workspace in local commands and CI.
- Actual rendered initial references are checked in a browser.
- Dependency-boundary verification covers the permitted reuse direction and
  rejects app or common-UI dependencies on the reference workspace.
- Existing frontend acceptance checks remain passing; no runtime services or
  secrets are introduced to verify this setup.

## Fixed Constraints

- Use an independent Storybook workspace package, reusing the existing repository
  toolchain and common UI where appropriate.
- Reference may depend on common UI; product apps may depend on common UI;
  common UI and product apps must not depend on reference.
- Product applications remain leaves: reference must not import their implementations.
- Current design choices are provisional. Approval of a reference does not by
  itself authorize promotion to common UI.
- Keep tracked artifacts in English and preserve existing product behavior.
- Deliver a focused pull request; merging and deployment require separate approval.

## Open Assumptions

None affecting the requested outcome. Package naming, file layout, and minimal
scaffolding fixtures are local implementation details, not new design decisions.

## Confirmation

The requester explicitly requested the described setup and ownership model on
2026-10-30 and, after receiving this complete Requirements document, instructed
execution without an additional approval round. The scope remains the previously
agreed structure, documentation, Agent guidance, and verification; no visual
design choice is approved by that instruction. The requester additionally
clarified on 2026-10-30 that a general reference representation may coexist with
a web-specific promoted common-UI implementation.
