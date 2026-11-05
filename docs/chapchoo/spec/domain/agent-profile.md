---
title: "Agent handles and saved profiles"
---

# Agent handles and saved profiles

An Agent has a Repository-owned opaque identity, an immutable public handle and
independently editable saved presentation. None replaces current Team membership
or private ownership authority.

## Handles and Team context

Final handles contain 3–16 lowercase ASCII letters, digits or hyphens, with
alphanumeric first and last characters. Consecutive hyphens are valid. Ingress
accepts the exact final value: it does not trim, lowercase, transliterate or
silently generate a different handle. Agent handles are installation-wide unique
among live Agents, independently of the live collaborative-Team handle namespace.
Personal Teams have `NULL` handles. Collaborative Team handles are immutable;
their settings update name only.

Repository scoped Agent lookup requires the exact current Team ID and handle,
then independently verifies membership and individual ownership. A handle's
uniqueness or knowledge of its value is not authorization. Foreign private
Agents remain not-found. Availability checks require current membership in the
selected Team and exact valid grammar, return only a boolean, and do not reserve
a handle. Final acceptance resolves races under the global live-Agent unique
constraint and reports a typed handle conflict.

## Profile values and catalogue

The saved Agent profile contains display name, catalogue image ID and one of six
palette keys: `butter`, `sage`, `apricot`, `teal`, `lilac`, `rose`. The handle is not
part of the profile update payload. Repository profile updates retain Agent,
Computer, Team, request/lifecycle, native resource and handle identities; they
change no native effect or runtime persona.

The authenticated AGENT catalogue stores Repository-issued image identities,
internal English names and local logical asset keys. Six builtin entries seed
once during preparation; subsequent preparation retains assignments and valid
installation extensions. Asset keys are 1–128 lowercase ASCII letters, digits or
hyphens, starting alphanumerically. They are logical local assets, not paths,
URLs, uploads or arbitrary external images. There is no catalogue management UI
or upload API. Catalogue pages use compact nullable keysets with bounded Store
limits 1–100; the public RPC accepts 1–20 and defaults to 20.

Migration initializes each existing Agent's profile independently from its
original Agent ID and persists it once. Later renaming, lookup and preparation do
not recompute an established image or palette. Collision-free initial handles
are assigned as part of the same explicit preserving transaction. See the
[identity cutover](repository-identity.md).

## Final creation and client candidates

Agents.Create requires the final Team, Provider, display name, handle, image ID
and palette along with its purpose-scoped Request ID. All those values enter the
creation fingerprint. Store validates catalogue/type and palette/handle grammar;
it neither picks defaults nor accepts a legacy missing-profile payload. Exact
accepted-request recovery returns the assigned current Agent or removed outcome,
including after profile edits or disconnection, without allocating again.

General creation's authenticated server snapshot reads the catalogue and draws
initial image and palette independently once. Hydration receives that exact draft;
changing Team discards another Team's draft. The browser previews the selected
image, pages the bounded catalogue and permits exact manual handle editing.

Client candidate generation transliterates the display name with pinned AnyAscii,
normalizes its proposal and adds a cryptographic base36 suffix. At most eight
availability attempts run. Generation/abort fences discard stale results; manual
edits survive name changes. Availability does not replace final uniqueness.
A final handle conflict starts a fresh bounded candidate resolution.

One complete creation payload and Request ID are frozen across uncertain
outcomes. While frozen, field edits and catalogue paging are refused; recovery
remains available despite later Provider/catalogue unavailability. Only a
positive pre-acceptance rejection for invalid input/transition, permission or
handle conflict releases that payload. Authentication, transport, unknown and
generic conflict failures are not evidence that the prior submission was absent.

Frozen-request recovery first reads owner/request-only history. A present result
returns the assigned identity and resolves the current saved Agent for
navigation, independently of a changed historical payload encoding. A removed
result cannot allocate a replacement. An absent observation is distinct from
cancellation or rejection: the original frozen payload/Request ID remains
unchanged if ordinary deduplicated acceptance is explicitly retried.

## Saved presentation and editing

List/detail/create/update projections carry the saved name, immutable handle,
catalogue reference, selected logical asset key and palette. Repository reads
join the selected asset directly, rather than having the browser scan the full
catalogue. Shared presentation resolves bundled artwork beneath the current
web mount; reads and navigation never choose new random values.
The selected palette supplies coordinated theme-aware surface, foreground and
emphasis colors on those profiles without changing workspace appearance.

Image choices are bounded catalogue-backed thumbnail buttons with a visible
selected check; colors are six named circular swatches, not text dropdowns.
Desktop uses a three-column image grid beside the preview. Mobile uses a compact
single row where six touch targets fit, with a narrower-width grid fallback.
Native button focus and Enter/Space activation retain keyboard access. Selection
updates the controlled preview without submitting. Explicit random-color actions
choose a different palette from the current draft; other input edits never call
that action or rerandomize the image.

Detail renders the saved profile and opens an independent controlled edit
draft in a desktop dialog or mobile bottom sheet. Name/image/palette changes
update its preview; the handle is read-only. Initial focus stays at the dialog heading so opening a narrow
editor does not scroll its preview out of view or immediately summon a keyboard.
The profile identity retains its existing alignment with a separate in-flow edit
target rather than overlapping long names. Catalogue pages are
supplementary: failure retains the chosen reference and name/color draft, not
a fabricated default. Save disables duplicate submission and submits the complete
selection. Failure retains the draft; Cancel, close and Escape discard it
without a mutation. Pending Save retains ownership and disables dismissal.
Confirmed success uses the saved response and invalidates related list/detail
queries. Background Computer observation does not overwrite in-progress edits.

## Connections

- [Repository identity](repository-identity.md) — IDs are opaque and separate from handles, request keys and native witnesses.
- [Private Agent ownership](agent-ownership.md) — context selection does not share an owner's Agent with another member.
- [Owner RPC](computer-client-rpc.md) — current public create, availability, catalogue and read contracts.
- [Computer web](computer-web.md) — creation presents final inputs and preserves uncertain submissions; native convergence remains backend-owned.

## References

- [`handle.rs`](../../../../rust/crates/domain/src/handle.rs)
- [`computers.rs`](../../../../rust/crates/persistence/src/computers.rs)
- [`profile_initialization.rs`](../../../../rust/crates/persistence/src/profile_initialization.rs)
- [`handle.ts`](../../../../typescript/packages/web-core/src/handle.ts)
- [`creationRequest.ts`](../../../../typescript/apps/web/src/features/computers/creationRequest.ts)
- [`computers.proto`](../../../../proto/chapchoo/computers.proto)
