---
title: "Account and Team transaction contracts"
---

# Account and Team transaction contracts

Account, session, membership and invitation state changes complete under current database authority before external delivery.

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
changes. It accepts the exact final immutable handle; personal Team handles are
null. Settings patch name only, without rewriting identity or public handle.
Only the owner appoints/demotes managers, transfers ownership or deletes;
the former owner is retained as Manager. Managers manage settings and ordinary
members but cannot remove managers/owner. Personal Team management is rejected.
Leaving or removing a member atomically accepts terminal deletion for that
member's private cloud Agents before removing membership. Collaborative Team
deletion accepts terminal deletion for all its Agents, immediately hides the
Team from lookup, selection and invitation discovery, and removes invitations.
The Team row and memberships remain only as operational cleanup bookkeeping
while Agent FKs still exist. A partial unique index enforces handle uniqueness
only for live Teams, so the deleted handle is immediately reusable under a new
Team ID without transferring any old Agent. After confirmed Computer cleanup removes the last
Agent, a separate post-commit collection removes that Team and cascades its
memberships. Empty Teams are collected immediately. Accounts, personal contexts
and historical Agent creation receipts survive. These internal rows are not
selectable archives and cannot be restored. If post-commit collection fails,
only the current owner may explicitly retry deletion against the retained pending
Team. That retry preserves the original deletion timestamp and terminal Computer
intent, then retries collection; other reads and mutations still hide the Team.
A newly created Team reusing its handle has independent identity and resources.

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

## Connections

- [Account schema and explicit preparation](account-storage.md) — The Store owns durable account identities, schema preparation and installation key binding. Completed account operations are described separately.
- [Account service operations and mail](account-operations.md) — Application operations prepare credentials, bound password work and submit optional mail outside database transactions.
- [Account RPC boundaries](account-rpc.md) — Account transport owns service admission, caller credentials and validated wire outcomes. Business transactions and process lifetime have separate owners.
- [Private Agent ownership and Team deletion](agent-ownership.md) — Team membership selects context while individual ownership remains the private Agent boundary. Context deletion accepts cleanup intent, not completed native deletion.
- [Web authentication, request ownership and cookies](web-session.md) — Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.

## References

- [`rust/crates/persistence/src/accounts.rs`](../../../../rust/crates/persistence/src/accounts.rs)
- [`rust/crates/persistence/src/sessions.rs`](../../../../rust/crates/persistence/src/sessions.rs)
- [`rust/crates/persistence/src/teams.rs`](../../../../rust/crates/persistence/src/teams.rs)
- [`rust/crates/persistence/src/challenges.rs`](../../../../rust/crates/persistence/src/challenges.rs)
