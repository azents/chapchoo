---
title: "Account Storage and Explicit Preparation"
created: 2026-10-29
tags: [accounts, rust, database]
spec_type: domain
domain: accounts
code_paths:
  - rust/crates/chapchoo-persistence/src/preparation.rs
  - rust/crates/chapchoo-persistence/src/store.rs
  - rust/crates/chapchoo-persistence/src/models.rs
  - rust/crates/chapchoo-persistence/src/identity.rs
  - rust/crates/chapchoo-persistence/src/accounts.rs
  - rust/crates/chapchoo-persistence/src/challenges.rs
  - rust/crates/chapchoo-persistence/src/sessions.rs
  - rust/crates/chapchoo-persistence/src/teams.rs
  - rust/crates/chapchoo-persistence/migrations/sqlite/00000000000001_accounts/up.sql
  - rust/crates/chapchoo-persistence/migrations/postgres/00000000000001_accounts/up.sql
last_verified_at: 2026-10-30
spec_version: 1
---

# Account Storage and Explicit Preparation

## Current Reachability

The persistence library exposes explicit preparation/readiness and completed
account/team/session operations with owned asynchronous pool leases. The
[RPC and operator composition](account-rpc.md) calls these library boundaries.

Preparation embeds separate SQLite/PostgreSQL migrations. PostgreSQL stores
native Team-kind, membership-role, grant-kind and invitation-status enums;
SQLite uses equivalent constrained text. Instants are signed Unix epoch
milliseconds in BIGINT columns, supplied by the application acceptance boundary.

## Keys and Relationships

| Table | Identity and constraints | References and deletion behavior |
| --- | --- | --- |
| installation_key_binding | Singleton 1; 32-byte derived key check | No root key stored; persistent across restart/migration |
| users | Stable text ID; unique canonical email key; protected password hash; boolean server-admin assignment | Retained account identity; no deletion capability implemented |
| teams | Stable ID; unique handle; one owner; unique personal Team per owner | Owner references users; composite same-Team owner membership is deferred NO ACTION |
| memberships | Composite Team/User primary key; Manager/Member only | Team deletion cascades memberships; User deletion is constrained |
| personal_contexts | One association per User, unique Team | Composite deferred membership reference; no automatic collaborative deletion of personal context |
| auth_sessions | Stable ID; unique current hash, indexed previous hash; fixed-size recovery nonce/ciphertext; validity/rotation/revocation facts | References User; User deletion constrained |
| grants | Stable ID; unique hash; signup or password reset; issued/expiry/consumption/revocation facts | Manual/invitation issuer references User; configured-mail signup can have no issuer; reset requires issuer and target User, signup has no target User; deletes constrained |
| invitations | Stable ID; Team, canonical recipient email; processing state/timestamp consistency | Team deletion cascades invitations; inviter User deletion constrained; recipient may be unregistered |
| login_challenges | Stable ID; unique binding hash, protected code verifier and expiry/consumption | Email-bound, no required recipient User; no stored raw code |
| initial_setup | Singleton 1 with consumption timestamp | Initial migration creates availability metadata only, not an administrator |

Foreign keys and schema checks do not replace the business acceptance
transactions. In particular, personal Team kind/owner consistency, permission
checks and atomic consumption are repository obligations rather than guarantees
of any arbitrary manual SQL statement.

## Preparation, Binding and Readiness

- SQLite preparation owns an advisory lock on a non-truncated database file.
  PostgreSQL owns a dedicated session advisory lock throughout all migrations
  and initial binding. Another preparation invocation fails visibly as busy.
- Existing binding is verified before migrations. The initially empty migration
  may be followed by a separately committed first binding; an interrupted empty
  installation can resume through explicit preparation, not normal startup.
- A missing binding with populated product tables fails as an integrity error.
  No command in this library replaces a bound key or clears user sessions.
- One key check is persisted in an owned transaction after initial migration.
  Preparation runs no network operation inside that transaction and creates no
  user, personal Team, password or login session.
- SQLite readiness opens the existing file in read-only URI mode; a missing file
  is not created. PostgreSQL uses a read-only transaction. Both validate the fixed
  key check and the expected migration-history identity without migration-harness
  helpers that create schema metadata.
- Default per-migration transactions can preserve an earlier committed migration
  when a later migration fails. These functions do not promise release rollback.

The operator preserves the database and original root together. Runtime
application startup invokes only readiness, not preparation. A
changed root, absent history or incorrect migration set does not authorize
adopting a new key, deleting data or creating the first administrator.

## Completed Operation Boundaries

Every asynchronous operation acquires its own official bb8 connection lease,
executes a complete DB-only transaction and returns detached typed values.
SQLite uses immediate write admission with foreign keys enabled on each
connection; PostgreSQL locks the actor/target User before mutable session or
credential state, and Team before membership changes. User state uses
`FOR NO KEY UPDATE`, preserving serialization without blocking the FK key-share
checks needed during an ownership transfer.

Password verification takes a detached credential snapshot, runs bcrypt outside
the transaction and returns an opaque `VerifiedCredential`. Session acceptance
locks/revalidates that same stored hash so reset cannot leave an old-password
session alive. Initial administrator creation and signup create the User,
password, personal Team/membership/association and initial session together.
Signup also consumes its grant in that transaction.

Session renewal uses the configured validity, rotation and previous-token
overlap, rejects expired/revoked state and returns one committed latest value
to concurrent previous-token holders. Unchanged and previous-token renewals
do not extend expiry. Logout revokes the corresponding session. Reset consumes
the intended User's recovery grant, updates its verifier and revokes all that
User's sessions together, without returning a new session.

Correct email-code verification consumes its challenge once. An unknown email
returns a committed registration-required outcome, not unrestricted signup.
Wrong codes neither consume nor extend a challenge. Creating a new request
retains other still-unconsumed, unexpired challenges. The service checks mail
configuration before email-only issuance and delivers after commit; the
repository sends no email and stores no raw code.

Collaborative Team creation atomically creates owner membership. Current
membership and owner reference determine permissions, including after concurrent
changes. Only the owner appoints/demotes managers, transfers ownership or deletes;
the former owner is retained as Manager. Managers manage settings and ordinary
members but cannot remove managers/owner. Personal Team management is rejected.
Deletion cascades the collaborative Team's memberships/invitations while
preserving accounts and personal contexts.

Invitations bind canonical recipient email and are processed exactly once by
that recipient. Acceptance creates Member membership; decline does not. An
unknown email with a prepared configured-mail signup draft creates grant and
invitation together; without that draft it creates only the pending invitation.
An already registered recipient never receives an extra signup grant from this
operation. Signup does not accept an invitation.

The operator recovery method promotes an existing User only. It has no network
authentication and must only be wired into the operator CLI capability, not a
public/general/Admin RPC. Connection/readiness authority is still required.

Grant revocation is scoped to its expected Signup/PasswordReset kind inside
the accepting transaction. Typed rejections distinguish mismatched recipient,
registered email, duplicate membership, processed invitation, owner-leave
restriction and unique Team-handle conflicts. Generic integrity/DB errors do not
become fabricated specific business outcomes.

## Verification Boundary

Actual temporary SQLite and private PostgreSQL tests exercised initial/repeat
preparation, same/wrong key behavior, read-only readiness, owner/membership
constraints and missing-binding rejection. A fixture also exercised resuming
explicit preparation after schema creation but before first key binding.

Both-engine operation tests also cover concurrent renewal/signup, old-password
login racing reset, grant consumption rollback after a failed session insertion,
code consumption, team role rules, duplicate acceptance, transfer racing target
leave, invitation/grant rollback and collaborative cleanup. Owned-future
cancellation tests verify connection replacement through a connection-local
marker, zero committed cancelled rows and a successful subsequent write.

These are library/integration tests, not public RPC/browser E2E, operator CLI
verification or Helm/Argo CD execution. Those surfaces remain pending until their
implementation and tests exist.
