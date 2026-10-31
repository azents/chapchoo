---
title: "Account Storage and Explicit Preparation"
created: 2026-10-29
tags: [accounts, rust, database]
spec_type: domain
domain: accounts
code_paths:
  - rust/crates/persistence/src/preparation.rs
  - rust/crates/persistence/src/store.rs
  - rust/crates/persistence/src/models.rs
  - rust/crates/persistence/src/identity.rs
  - rust/crates/persistence/src/accounts.rs
  - rust/crates/persistence/src/challenges.rs
  - rust/crates/persistence/src/sessions.rs
  - rust/crates/persistence/src/teams.rs
  - rust/crates/persistence/src/display.rs
  - rust/crates/domain/src/display.rs
  - rust/crates/persistence/migrations/sqlite/00000000000001_accounts/up.sql
  - rust/crates/persistence/migrations/postgres/00000000000001_accounts/up.sql
  - rust/crates/persistence/migrations/sqlite/00000000000002_display/up.sql
  - rust/crates/persistence/migrations/postgres/00000000000002_display/up.sql
last_verified_at: 2026-10-31
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
| users | Stable text ID; unique canonical email key; protected password hash; boolean server-admin assignment; complete language/time-zone/region | Retained account identity; no deletion capability implemented |
| display_defaults | Singleton 1; complete language/time-zone/region | Installation fallback and account-creation defaults, distinct from personal settings |
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
- Existing binding is verified before migrations. Migrations, first key binding,
  display-default initialization and missing-user backfill commit together under
  that lock. A failed preparation rolls back that invocation's schema/data
  changes; explicit preparation can be rerun.
- A missing binding with populated product tables fails as an integrity error.
  No command in this library replaces a bound key or clears user sessions.
- Preparation detects its environment before the transaction and runs no network
  operation inside it. It creates no user, personal Team, password or login session.
- SQLite readiness opens the existing file in read-only URI mode; a missing file
  is not created. PostgreSQL uses a read-only transaction. Both validate the fixed
  key check, exact expected migration set, complete singleton and absence of
  incomplete user display markers, without creating schema metadata.

Display defaults initialize once from the first successful preparation machine:
its IANA time zone and supported environment locale, with English/en-US for an
unsupported locale. Undetectable/invalid machine time zones fail preparation when
initial values are needed. Bundled Jiff timezone data validates names without a
host tzdata prerequisite. Existing users' missing time zones use the current
preparation machine, not an edited installation default; missing language/region
use installation defaults. Repreparation preserves established values.
Invocation atomicity does not promise rollback of earlier deployed releases.

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
Each creation input can omit display dimensions; current defaults resolve those
dimensions inside that same transaction. Personal display updates authenticate
and lock the current User, patch only supplied dimensions and return the committed
complete User. Concurrent independent patches preserve unrelated fields. Admin
default updates authenticate current administrator authority and lock the
singleton; they never update existing users. A public defaults read exposes only
the three validated presentation values.

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

Display upgrade tests on both engines start with populated version-1 data,
exercise failed migration rollback and successful retry, preserve existing
account/team/session data, and reject incomplete readiness. Display operation
tests cover independent patches, permissions and creation-time defaults.
These library tests are distinct from the public RPC/browser evidence in the
corresponding Specs and do not establish Helm/Argo CD execution.
