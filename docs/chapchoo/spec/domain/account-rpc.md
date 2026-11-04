---
title: "Account RPC boundaries"
---

# Account RPC boundaries

Account transport owns service admission, caller credentials and validated wire outcomes. Business transactions and process lifetime have separate owners.

## Composition and authority

The `chapchoo` binary composes configuration, one authoritative Store, bounded
password work, optional SMTP delivery and application services.
RPC handlers do not depend on the DB implementation. `AccountServices` composes
separate domain adapters over shared account operations and admission, without
one transport handler implementing unrelated domain traits.
Authentication and PersonalPreferences are shared Client/Admin dependencies.
AccountLifecycle, TeamManagement and InvitationManagement are registered on
the Client group; Installation, AccountGrants, DisplayDefaults and AdminWorkspaces are
registered on Admin. Groups can run independently or as one deduplicated union
on a single listener. Admin business calls
recheck current server-administrator authority in the repository operation.

AdminWorkspaces exposes ListTeams, CheckHandle and CreateTeam using the ordinary Team
messages. It lists the current administrator's accessible live Teams, not all
installation Teams; creation grants the caller ordinary owner membership. The
same storage transaction authenticates and checks current administrator status
before the read or write. It grants no member management or private-Agent access.
TeamManagement remains Client-only. Admin web uses its Admin backend exclusively.
TeamManagement.ResolveHandle returns only a currently accessible live
collaborative Team. Its CheckHandle authenticates the caller and returns a
boolean without revealing existing Team ownership or membership. AdminWorkspaces
availability additionally rechecks current installation-admin status in the same
Repository operation. Availability is not reservation; final creation owns
uniqueness.

Protected calls carry exactly one `authorization: Bearer <opaque-token>`
metadata value. Each call parses its own credential. Channels do not hold a
mutable caller identity, and ordinary accounts can use Authentication on the Admin listener
without gaining Admin business privileges. Logout and password reset affect the
shared session authority on both listeners.

`prepare` explicitly creates/migrates schema and establishes/verifies the fixed
key binding. `serve` only checks readiness, never migrates. `grant-admin --email`
promotes an existing account directly under local DB/key authority and is not
registered as an RPC. First-admin initialization instead requires the configured
setup proof and the transaction's empty/unconsumed setup state.

## Protocol

Protobuf is the single message/service source. Rust builds Tonic/Prost code with
locked vendored protoc. The committed server-side Node package uses ts-proto and
grpc-js, ESM imports, `bigint` int64 values and Timestamp seconds/nanos. Generated
Rust message Debug emits no fields. Regeneration checks compare complete output.

Ingress rejects missing/invalid email, password, proof, ID, pagination and enum
values. Team snapshots have nullable personal handles; collaborative handles are
required and immutable. Team update accepts optional name only: omission preserves
the name and a present empty value fails.
RPC message decoding is bounded to 64 KiB. Instants are Protobuf Timestamp and
map to signed epoch milliseconds at the storage boundary. Unexpected enum values
remain wire values until validated; they never become success/default roles.

Public login/signup/initialization/renewal return the dedicated common session
result. Identity and domain reads omit bearer secrets. Code issuance returns
binding/expiry only. Proof previews expose only usability and, when usable,
masked recipient hint/expiry. Grant inventory omits proof/URL; initial issuance
returns its new proof and general-web URL.

Every User snapshot includes complete independent display language (`en`, `ko`,
`ja`, `fr`), IANA time zone and regional format (`en-US`, `ko-KR`, `ja-JP`,
`fr-FR`). PersonalPreferences.DisplayDefaults publicly reads only installation presentation
defaults. PersonalPreferences.UpdateDisplay authenticates the caller, applies an independent
partial patch and returns the committed User. DisplayDefaults.UpdateDisplayDefaults
rechecks current administrator authority and returns the committed defaults.
Signup and initialization accept optional partial display values, resolving
omitted dimensions from current defaults in the creation transaction. Invalid
values fail with INVALID_ARGUMENT; omission means unchanged for patches and
current installation fallback for creation. There is no automatic mutation replay.

Canonical generated reason names are carried in the ASCII trailing metadata
field `chapchoo-domain-reason`. Credential/session/code rejection is
UNAUTHENTICATED; permission is PERMISSION_DENIED; invalid input/proof/password is
INVALID_ARGUMENT; duplicate identity/member/handle is ALREADY_EXISTS; invalid
transitions/processed invitation/owner leave are FAILED_PRECONDITION; hidden
targets are NOT_FOUND. Missing mail is FAILED_PRECONDITION; saturated password
work is RESOURCE_EXHAUSTED. Infrastructure failures retain sanitized failure
status and never imply rollback or authorize mutation replay. Missing/unknown
reason metadata remains a generic transport failure for clients.

## Verification boundary

Rust unit/integration suites cover credential admission/cancellation, optional
mail and committed failure, and actual selected-union authorization/transport calls.
The root interoperability gate launches disposable CLI instances for both SQLite
and PostgreSQL and calls them using generated Node clients. CLI tests cover
explicit preparation, readiness failure, local recovery, common Authentication and owned
termination/partial-startup cleanup. SMTP tests exercise a local protocol fixture,
single-attempt success/failure, required STARTTLS without downgrade, and rejection
of an untrusted TLS certificate, not an external mailbox.

The backend tests do not establish packaged browser or cluster acceptance.
Those verification boundaries are recorded in the
[web](account-web.md) and [delivery](account-delivery.md) Specs.

## Connections

- [Account and Team transaction contracts](account-transactions.md) — Account, session, membership and invitation state changes complete under current database authority before external delivery.
- [Credential primitives](credential-primitives.md) — Reusable domain and crypto values protect representation and secrecy; service and persistence owners supply admission, expiry and installation binding.
- [Web authentication, request ownership and cookies](web-session.md) — Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.
- [Foreground process ownership and health](runtime-process.md) — The composition root owns admission, startup and shutdown of its selected backend and web processes. A successful cleanup submission is not completed shutdown.

## References

- [`proto/chapchoo/accounts.proto`](../../../../proto/chapchoo/accounts.proto)
- [`rust/crates/rpc/src/adapters`](../../../../rust/crates/rpc/src/adapters)
- [`rust/crates/rpc/src/groups.rs`](../../../../rust/crates/rpc/src/groups.rs)
