---
title: "Private Agent ownership and Team deletion"
---

# Private Agent ownership and Team deletion

Team membership selects context while individual ownership remains the private Agent boundary. Context deletion accepts cleanup intent, not completed native deletion.

## Team Scope and Context Deletion

Migration 5 attaches every pre-existing Agent and creation receipt to its owner's
existing `personal_contexts` Team, preserving Agent/Computer identities, names,
execution and resource records. A missing mapping fails and rolls back the
whole migration; rows are never dropped or assigned to an arbitrary Team.
The down migration rejects erasing this required context.

Team handles remain unique among live Teams, not hidden cleanup rows. Marking a
Team deleted releases its public handle immediately, preserving prior deletion
behavior while retaining its exact Team/Agent cleanup identities. Reusing the
handle creates a new Team ID and cannot adopt old Agents or receipts.
SQLite rebuilds the closed Team/membership/personal-context/invitation FK group
inside the same preparation transaction, with foreign keys enabled and every
existing row copied before replacement. PostgreSQL replaces the global handle
constraint with the equivalent live-Team partial unique index. Both-engine
tests verify row preservation and reuse while the deleted Team still has Agents;
the PostgreSQL catalog verifies the unique-index predicate.

Owner operations lock the selected Team and validate current membership before
Provider and Computer admission. Detail, retry, lifecycle and force removal
resolve the owned Agent's actual Team rather than trusting a caller's Team ID.
Private dispatch revalidation also requires an existing Agent and membership;
trusted Provider reconciliation remains independent of user membership.

Team leave/kick records deleting intent for only the departing owner's cloud
Agents, revokes Connector authority and then removes membership in the same
transaction. Collaborative Team deletion applies that intent to all Agents.
Context-driven Request IDs derive deterministically from the exact Team, Agent
and predecessor request; an already-deleting Computer keeps its terminal request.
All paths use the same never-prepared absence proof and resource cleanup gates
as direct deletion. There is no automatic move to the personal Team.

`teams.deletion_requested_at` marks an internal pending deletion. Such Teams are
excluded from Team lookup/list/admission and invitation discovery immediately.
The row is retained only because Agents still reference it; it is not an archive
or restore capability. Empty pending Teams are physically deleted immediately.
Confirmed cleanup, force removal and completed-inventory collection invoke Team
collection in a separate transaction after Provider/Computer locks are released,
preserving Team → sorted Providers → Computers lock order. Unresolved external
resources have no TTL, including after their user's membership ends.

Both-engine tests verify private same-Team isolation, cross-Team admission and
receipt conflict, leave/kick/delete cleanup, background access after departure,
concurrent creation versus Team deletion, preserving migration and atomic
missing-map rollback. The migrated PostgreSQL catalog verifies the required
`agents.team_id` and its non-cascading Team FK.

## Operations and Permission Boundary

Every owner read/mutation executes within the existing complete DB operation and
revalidates the account session after its User lock. Foreign private Agents return
not-found. Provider registration accepts a `VerifiedSystemOwner`, which can only
be constructed by a configured verifier checking both credential digest and an
explicit allowed Provider identity. An asserted owner name or Provider key is not
that proof. Existing Provider ownership cannot be overwritten by another owner.
This verifier neither generates configuration credentials nor treats the
installation encryption root as authentication.

Cloud creation locks the current owner and selected Team, checks membership and
its owner-scoped creation receipt,
then validates an authorized currently connected creation-capable Provider and
writes the receipt, Computer/running intent and Agent atomically. Exact replay
returns its assigned Agent/current state even if Provider connectivity has since
changed. Reuse with a changed Team or canonical name/Provider payload rejects. Replay after
Agent removal returns removed, never a fresh allocation. External provisioning is
outside this transaction.

Available Provider queries require eligible current Control/component connection,
current or still-authorized immediately previous credential, nonrevoked/nonexpired
Provider key and creation capability. Keys/endpoint/resource UIDs are not in
choice results. Queries and Agent lists use bounded keyset pagination. No request
queues or Kubernetes calls execute in these DB operations.

Canonical payload fingerprints include operation and length-delimited typed field
boundaries. They do not serialize arbitrary JSON or retain Agent content in the
creation receipt. Stored credential digests and payload hashes are not bearers;
Debug for owner verifier/proof, private rows, drafts and projections is redacted.

## Application Service

`ComputersService` generates Agent, Computer, execution and session IDs and all
new component credentials server-side. Caller Request IDs remain stable for
deduplication. Its system-owner verifier is supplied by trusted composition, not
constructed from request owner assertions. System metadata/key methods verify
configured credential and Provider scope; component keys cannot self-rotate.
User-owner methods use current account authority. Store validation and locked
admission remain authoritative for every generated draft.

Credential methods use execution-scoped sessions and distinct bootstrap versus
operational roles. Key recovery returns the exact current result and expiry,
not a new generated draft. The service also exposes typed owner reads/lifecycle
admission and internal current-source inventory/evidence methods. It does not
activate component listeners, configure deployment credentials, create a
reconciler or perform external resource effects.

## Connections

- [Account and Team transaction contracts](account-transactions.md) — Account, session, membership and invitation state changes complete under current database authority before external delivery.
- [Computer schema and native identities](computer-storage.md) — Durable Computer state records authority, exact resource identity and cleanup obligations; schema records do not themselves perform external effects.
- [Computer owner and installation inventory RPC](computer-client-rpc.md) — Owner operations and redacted installation inventory are separate capabilities over current account authority.
- [Computer lifecycle admission and Provider keys](computer-lifecycle.md) — Owner requests commit durable intent under current authority. Component evidence and external completion determine what may happen next.
- [Workspace context and navigation](workspace-navigation.md) — Navigation selects an accessible context without granting access to another owner’s private Agents.

## References

- [`rust/crates/persistence/src/computers.rs`](../../../../rust/crates/persistence/src/computers.rs)
- [`rust/crates/persistence/src/agent_cleanup.rs`](../../../../rust/crates/persistence/src/agent_cleanup.rs)
- [`rust/crates/service/src/computers.rs`](../../../../rust/crates/service/src/computers.rs)
- [`rust/crates/persistence/src/teams.rs`](../../../../rust/crates/persistence/src/teams.rs)
