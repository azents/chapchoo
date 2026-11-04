---
title: "Agent Profiles and Team-Scoped Handles Requirements"
created: 2026-11-04
tags: [agents, accounts, frontend, identity]
document_role: primary
document_type: requirements
snapshot_id: profile-261104
---

# Agent Profiles and Team-Scoped Handles Requirements

- Snapshot: `profile-261104`
- Document reference: `profile-261104/REQ`

## Problem

Users cannot give an Agent a persistent visual identity or edit its profile after
creation. Agent detail addresses expose internal identifiers rather than a useful
name, and Team addresses do not use their existing handles. The creation and
navigation experience must distinguish display names, permanent handles and
stable resource identity without requiring repeated manual naming.

## Primary Context

### Primary Actor

An authenticated user creating and managing their own Agents in an accessible
personal or collaborative Team.

### Primary Scenario

The user opens an accessible Team's Agent list, creates an Agent with a profile
and an automatically proposed handle, reaches that Agent through a Team-scoped
address, and later edits its visible profile without changing the Agent's handle,
identity, Computer association or ownership.

## Supporting Scenarios or Effects

- A user creates a collaborative Team with an automatically proposed permanent
  handle, selects it and reaches its initially empty Agent list.
- A user switches between personal context and accessible collaborative Teams.
- Existing Agents receive usable profiles and handles without resource recreation.
- Product-issued identities become consistently represented across account,
  Agent and operational boundaries while preserving their existing relationships.

## Goals

- Recognizable, persistent Agent profiles across list, creation, detail and editing.
- Automatic usable handle proposals with explicit correction before creation.
- Permanent handles and Team-qualified Agent addresses.
- Personal navigation resolved through the authenticated user's personal context.
- Preserved private ownership, lifecycle controls and existing data continuity.

## Non-Goals

- Personal-computer enrollment or selection, harness integration, creation-only
  persona presets, user-facing cloud addition, or richer cloud metadata work.
  These remain deferred capabilities rather than removed product directions.
- Mention parsing, reference storage, rendering, notifications or chat features.
- Arbitrary profile uploads, external image sources, custom color values,
  catalogue-management mutation tools or their Admin screens.
- Dedicated profile-edit locks, conflict-resolution or merge interfaces.
- Team-public Agents or broader access for Team managers or installation Admins.
- New redirect archives, alternate-ID compatibility modes or identifier-history
  features without an explicit subsequent decision.

## Requirements

### REQ-1. Recognizable Agent list and detail

List and detail present the same saved profile identity.

**Acceptance criteria**

- Profile image and display name dominate each list card; handle is secondary
  information beneath the name, not a replacement for it.
- Desktop uses two list columns and mobile one, with long text contained.
- The Agent's selected palette supplies coordinated surface, text and emphasis
  colors without changing global workspace appearance.
- Detail retains the profile area, profile-edit entry, Computer controls,
  collapsed diagnostics and separately placed deletion action.
- Unimplemented harness information or persona features are not presented as
  available capabilities in this delivery.

### REQ-2. Complete creation in the current capability scope

Create an Agent in one screen using currently supported Providers and the chosen
profile and handle.

**Acceptance criteria**

- The screen includes eligible existing cloud Provider selection, profile image,
  name, handle, palette and a live profile preview.
- Desktop places preview and input beside one another; mobile stacks them.
- Initial image and palette are independently selected at random once per
  creation flow. Editing other inputs does not rerandomize them.
- Users can select another image or palette and explicitly rerandomize the color.
- Personal-computer, harness, persona and cloud-addition controls are omitted.
- Successful creation reaches the accepted Agent's Team-qualified handle address.
- Existing eligibility, creation-receipt and uncertain-result recovery behavior
  remains intact. A lost response does not become a fresh creation attempt.

### REQ-3. Persistent profile editing

Users can edit an existing Agent's image, display name and palette.

**Acceptance criteria**

- Editing opens with current values, in a desktop dialog or mobile bottom sheet.
- Changes update a draft preview; Save applies them and Cancel, close or Escape
  discards them.
- Reopening retains saved choices rather than rerandomizing them.
- Basic progress and error feedback preserves the draft on a failed save.
- Editing display name, image or palette never changes the handle.
- Computer and execution configuration are not profile-edit fields.

### REQ-4. Initial catalogue and palette availability

Provide the agreed initial selection while retaining the ability to extend the
character catalogue later.

**Acceptance criteria**

- The initial image catalogue contains the six existing bundled characters.
  This is an initial offering, not a permanent six-item product limit.
- The catalogue can be registered, read, selected and referenced by Agents;
  management mutation tools remain outside this delivery.
- The initial named palette choices are butter, sage, apricot, teal, lilac and rose.
  Each represents a coordinated color set rather than an arbitrary color input.
- Existing Agents receive a deterministic initial character and an independently
  determined initial palette once. Saved choices survive catalogue growth,
  reordering, repeated reads and later user edits.

### REQ-5. Common permanent handle contract

Collaborative Team and Agent handles use the same syntax and become immutable
after creation.

**Acceptance criteria**

- Handles contain 3 through 16 characters, using lowercase ASCII letters, digits
  and hyphens. First and last characters are a lowercase letter or digit.
- Agent handles are unique installation-wide in the current delivery.
- Collaborative Team handles remain unique among live Teams.
- Users may correct a proposed handle before creation; no post-creation handle
  edit is available. Display-name editing remains independent.
- Directly supplied invalid or occupied handles are reported rather than silently
  rewritten. Final uniqueness is enforced when the resource is accepted.
- Personal Teams have no handle: the value is null, not an empty string or a
  synthesized personal handle.

### REQ-6. Automatic handle proposals without naming friction

Agent and collaborative Team creation propose handles from the display name and
automatically resolve collisions for automatically generated candidates.

**Acceptance criteria**

- Original multilingual display names remain unchanged.
- Automatic conversion produces a candidate within the common syntax and length
  bounds, with an availability check and visible feedback.
- If a converted candidate is shorter than three characters, preserve its usable
  part and supplement it with a random suffix.
- If conversion produces no usable text, use the resource-kind base `agent` or
  `team` and a random suffix.
- If an automatic candidate is occupied, add a random suffix, shorten the base
  as needed to remain within 16 characters, and check again.
- Manual candidate edits are retained when the display name subsequently changes;
  validation informs the user instead of replacing the manual candidate.
- An availability check is advisory, not a reservation or proof that a later
  create operation cannot encounter a conflict.

### REQ-7. Team-qualified navigation

Public Agent addresses always include a Team context and the Agent handle.

**Acceptance criteria**

- Collaborative Team addresses use the Team handle rather than its internal ID.
- Agent addresses are below that Team context, using the Agent handle even while
  Agent handles remain globally unique.
- Team context participates in target resolution; a mismatching Team cannot
  select an Agent solely because its handle exists elsewhere.
- Personal addresses use a fixed route alias resolved through the authenticated
  user's personal Team. The alias is not a stored Team handle.
- The address structure accommodates a later Team-scoped Agent-handle policy
  without implementing that uniqueness change now.
- Installation Admin pages are outside the public-web alternative-key review;
  this does not exempt their DTOs from the accepted general identity boundary.

### REQ-8. Team creation and selection flow

Retain the existing workspace picker and creation entry while reflecting permanent
handles.

**Acceptance criteria**

- The picker separates personal context from collaborative Teams and places
  Create Team below the Team choices.
- Collaborative choices show display name and handle; personal context shows
  its name without an invented handle.
- Creation shows name, automatically proposed editable handle, length feedback,
  the prospective Team address and the post-creation immutability notice.
- Cancel changes neither membership nor current context.
- Confirmed creation selects the new Team and opens its empty Agent list; the
  new Team also appears selected in the picker.
- Desktop popover and mobile navigation Drawer preserve existing interaction,
  keyboard and focus ownership.

### REQ-9. Existing data continuity

Initialize new identity and profile fields without discarding resources or
changing their effective ownership and relationships.

**Acceptance criteria**

- Existing Agents receive a handle once from their current name, applying the
  same conversion, bounds, fallback and collision rules as new candidates.
- The initial handle remains unchanged after initialization and later renaming.
- Existing overlength collaborative handles are shortened to fit; collisions
  are resolved by random regeneration rather than deleting either Team.
- Existing personal-Team handles become null while personal association remains.
- Agent, Computer, Team, user and operational relationships are preserved during
  identity transitions. Native resources and saved data are not recreated merely
  to change an internal identifier or its representation.
- Successful preparation leaves complete usable values; failed transition and
  retry behavior must be explicit in the implementation Design.

### REQ-10. Preserve authority, lifecycle and identity consistency

The new presentation and identifiers retain current product authority and
operational meaning.

**Acceptance criteria**

- Team membership and individual Agent ownership continue to control access.
  Handle knowledge or global uniqueness grants no discovery or access privilege.
- Existing start, stop, restart, safe preparation retry, deletion, force removal,
  delayed/uncertain states and independent cleanup responsibilities remain usable.
- Product-issued identifiers have a consistent lossless representation across
  affected repository consumers, without introducing a second independent
  identity for an existing resource.
- Authentication secrets and hashes retain their distinct security contracts;
  native/provider-assigned identifiers are not silently treated as product IDs.
- Existing server rendering, localization, appearance, request recovery and
  bounded observation contracts remain authoritative unless explicitly changed
  by the requirements above.

## Fixed Constraints

- PostgreSQL and SQLite remain supported persistence backends.
- Existing identity-representation and repository-boundary decisions remain
  binding technical inputs; their concrete mechanisms belong in the snapshot ADR
  and Design, rather than being new alternatives in this Requirements document.
- The initial catalogue and palettes reuse the existing artwork and agreed visual
  language. Profile choices are not tied to a persistent persona.
- Creation-time handle generation is distinct from one-time initialization of
  existing records. Their different execution contexts must not silently change
  normal creation behavior.
- Existing permission checks and truthful Computer-state projections are preserved.

## Open Assumptions

- `my` is the illustrative personal route alias; its exact spelling, route
  reservation and final path layout must be closed in the implementation Design.
- Exact random-suffix construction, migration ordering and failure/retry mechanics
  are not defined by the example handles.
- The deployed data volume and presence of legacy handles have not been inspected;
  transition behavior cannot assume the database is empty.
- Old-address handling and the transition of live operational identities require
  explicit design resolution. This document adds no implicit alias retention,
  credential reset or operational restart permission.

## Confirmation

The requester confirmed proceeding through technical design against this complete
Requirements snapshot on 2026-11-04, following presentation of the consolidated
requirements. This confirmation preserves the previously accepted product and
technical constraints. It does not authorize implementation, merge, deployment or
new unapproved material mechanisms.
