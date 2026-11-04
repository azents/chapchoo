---
title: "Agent Profile and Handle Design Decisions"
created: 2026-11-04
tags: [agents, accounts, architecture, identity]
document_role: primary
document_type: adr
snapshot_id: profile-261104
---

# Agent Profile and Handle Design Decisions

- Snapshot: `profile-261104`
- Requirements: [profile-261104/REQ](../requirements/profile-261104-agent-identity.md)
- Mode: Collaborative; the requester owns new material decisions.

This document records explicit constraints already selected by the requester.
It does not turn design investigation or recommendations into acceptance. The
requester confirmed the consolidated Requirements before this ADR was created.

## D1. UUIDv7 is the product-issued entity identity standard

- State: accepted, requester-selected constraint, 2026-11-04.
- Requirements: REQ-9, REQ-10.

Use UUIDv7 for product-issued entity identities across the product, not only
Agents. Authentication secrets, native-system identity and purpose-specific
request-correlation contracts remain different concepts. Do not shorten a
credential hash merely because an entity identifier becomes smaller.

UUIDv7 was selected over retaining the random 256-bit hex identity and compact
UUIDv4 alternatives. It supplies a standard 128-bit identity with time-local
insertion characteristics. Its timestamp prefix is not hidden. The exact database
representation and existing-identity transition remain to be resolved explicitly.

## D2. Repository owns the entire identity representation boundary

- State: accepted, requester-selected constraint, 2026-11-04.
- Requirements: REQ-7, REQ-10.

Every product entity ID exposed in a Repository DTO uses the entity prefix plus
the same UUIDv7 encoded as unpadded Base64URL. Prefixes consist of one or two
uppercase ASCII letters. Their selection is delegated to the designer unless
specified by the requester; requiring a third letter returns to the requester.

Only the Repository knows prefix mapping, encoding and reverse conversion.
Service, transport, browser and native adapters treat the completed value as an
opaque identifier. IDs never exposed outside the Repository need no prefix.
This rejects URL-only formatting and Service-side prefix construction.

This is one identity with two representations, not an independently generated
external key. Formatting does not replace handle-based public navigation.

## D3. Client-owned handle generation with authoritative acceptance

- State: accepted, requester-selected constraint, 2026-11-04.
- Requirements: REQ-5, REQ-6, REQ-8.

For normal Agent and collaborative Team creation, the client uses AnyAscii to
convert the display name, checks availability with the server, supplements
short/empty candidates and resolves automatic collisions with a random suffix
and another check. A manual override is retained. The server validates the final
supplied value and enforces uniqueness; it does not generate or rewrite it.

The constraints are lowercase ASCII letters/digits/hyphens, alphanumeric ends,
3..16 characters and no handle changes after creation. Team and Agent namespaces
remain separate. Agent uniqueness is installation-wide; collaborative Team
uniqueness retains the live-Team boundary. Personal Team handle is null.

The requester selected client-side generation over the discussed server-side
candidate generation. One-time initialization of existing Agents from their names
was separately approved; that exception does not move normal generation to the
server. Exact migration composition is not decided by this exception.

## D4. Team context owns public Agent navigation

- State: accepted, requester-selected constraint, 2026-11-04.
- Requirements: REQ-7, REQ-8.

Use collaborative Team handles in Team URLs and Agent handles below their Team.
Resolve personal context from the logged-in identity using a fixed path alias;
the alias is not the personal Team's null handle. Exact route layout is a Design
proposal until its routing and compatibility implications are resolved.

This rejects the proposed standard-ID Team path and a standalone global Agent
handle address. Team context must be real target scope, accommodating a possible
later Team-local Agent uniqueness policy without implementing that policy now.
Existing access checks remain independent of handles and navigation.

## D5. Separate image catalogue and product palette authorities

- State: accepted, requester-selected constraint, 2026-11-04.
- Requirements: REQ-3, REQ-4.

Images belong to an extensible installation catalogue starting with six fixed
builtins. Builtin artwork uses a stable logical asset key, distinct from the
catalogue's English internal name and installation-specific URL or physical path.
Initial delivery supports registration/read/selection/Agent reference, but not
catalogue mutation management, uploads or arbitrary external sources.

Colors use product palette keys for coordinated color sets, initially butter,
sage, apricot, teal, lilac and rose. They are not a managed installation palette
or arbitrary HEX values. Image and palette selection are independent.

Existing Agent defaults are chosen once using independent ID-based hashes over
a fixed initial ordering, stored as the actual selected values and not recomputed
when the catalogue changes. Exact hash construction remains a local design detail
subject to preservation of that behavior.

## D6. Current creation capability is a single screen

- State: accepted, requester-selected constraint, 2026-11-04.
- Requirements: REQ-2.

Present existing Provider selection and the approved profile/handle fields on one
screen. Keep desktop preview/input columns and mobile vertical ordering.
Exclude the deferred personal-computer, harness, persona and cloud-addition
controls. The larger staged target journey remains deferred, not deleted.

## Investigation preceding D7

The new identity format crosses durable and external boundaries in the current
system, not only database columns. Source investigation established:

- Account and component recovery ciphertext authenticates the exact session ID.
- Creation receipts retain a payload hash rather than the original complete
  creation payload; removed receipts retain an assigned Agent ID without its row.
- Native resources contain exact Computer/execution ownership labels or
  annotations. Docker volume/container labels cannot be treated as freely mutable.
- Running Connectors introduce themselves using a previously issued session ID.

Re-keying every database row alone cannot preserve these contracts. A safe
rollout requires a selected policy for live old-format identity use and retained
historical evidence. It must not silently accept raw old IDs, invalidate sessions,
recreate native resources, weaken ownership evidence, or assume no installed data.

The recommended candidate was a coordinated maintenance-window transition with
explicit native ownership-evidence preservation and credential-envelope
rewrapping. It remained pending during investigation and was subsequently
accepted by the implementation instruction recorded below.

## D7. Compact storage and coordinated preserving transition

- State: accepted, requester implementation instruction, 2026-11-04.
- Requirements: REQ-9, REQ-10.

The requester instructed implementation immediately after the complete candidate
and its operational/compatibility consequences were presented. Implement the
candidate using PostgreSQL native UUID and SQLite BLOB16, repository-owned UUID
issuance, explicit preparation under the existing installation binding, atomic
reference conversion and preserved credential envelopes/native allocation proof.

The release transition requires a maintenance window: new mutations/dispatch
stop, old executions retire safely while saved environments remain, and matching
new processes resume. Old clients/mutation formats and old ID URLs are not mixed
indefinitely with the new contract. This is design authority for the software and
operator procedure, not authorization to execute maintenance or migrate a deployed
database in this Session.

Preserve exact native names, UIDs, labels and saved-volume witnesses as allocation
evidence, separate from new entity identities. Keep historical receipt hashes;
read accepted/removed outcomes using authenticated owner and original request ID,
without replaying old mutation payloads. Rewrap authenticated recovery envelopes
with the original root and unchanged tokens/validity instead of resetting login
sessions or component credentials. Preparation failures roll back and fail closed.

Preserve configured Provider registration keys as explicit external registration
identifiers distinct from their canonical entity IDs; existing verified-owner
admission remains authoritative. Do not introduce a generic legacy-ID alias
registry or a compatibility mode. A downgrade after new-format writes rejects.

## D8. Unambiguous personal and Agent route segments

- State: accepted as part of the presented implementation baseline, 2026-11-04.
- Requirements: REQ-7, REQ-8.

Use `/teams/{teamHandle}/agents/{agentHandle}`, with `my` selecting the current
user's personal context. `my` is outside the 3..16-character handle namespace.
Agent creation uses `/teams/{teamHandle}/new-agent`, outside the Agent-handle
segment; ordinary valid handles such as `new` need no artificial reservation.
Team settings use `/teams/{teamHandle}` and lists `/teams/{teamHandle}/agents`.
No independent Team/Agent identifier is inferred from route spelling.
