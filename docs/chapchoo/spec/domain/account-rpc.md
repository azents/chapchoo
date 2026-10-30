---
title: "Account RPC and Foreground Backend"
created: 2026-10-30
tags: [accounts, rust, grpc, cli]
spec_type: domain
domain: accounts
code_paths:
  - proto/chapchoo/accounts.proto
  - rust/crates/chapchoo-protocol
  - rust/crates/chapchoo-service
  - rust/crates/chapchoo-rpc
  - rust/crates/chapchoo-mail
  - rust/apps/chapchoo
  - typescript/packages/chapchoo-rpc-client
last_verified_at: 2026-10-30
spec_version: 1
---

# Account RPC and Foreground Backend

## Composition and authority

The `chapchoo` binary composes configuration, one authoritative Store, bounded
password work, optional SMTP delivery and application services.
RPC handlers do not depend on the DB implementation. Auth is registered on both
listeners; General and Admin business services are registered only on their
corresponding listener. Either role can run independently. Admin business calls
recheck current server-administrator authority in the repository operation.

Protected calls carry exactly one `authorization: Bearer <opaque-token>`
metadata value. Each call parses its own credential. Channels do not hold a
mutable caller identity, and ordinary accounts can use Auth on the Admin listener
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
values. Optional Team name/handle preserve omission; present empty values fail.
RPC message decoding is bounded to 64 KiB. Instants are Protobuf Timestamp and
map to signed epoch milliseconds at the storage boundary. Unexpected enum values
remain wire values until validated; they never become success/default roles.

Public login/signup/initialization/renewal return the dedicated common session
result. Identity and domain reads omit bearer secrets. Code issuance returns
binding/expiry only. Proof previews expose only usability and, when usable,
masked recipient hint/expiry. Grant inventory omits proof/URL; initial issuance
returns its new proof and general-web URL.

Canonical generated reason names are carried in the ASCII trailing metadata
field `chapchoo-domain-reason`. Credential/session/code rejection is
UNAUTHENTICATED; permission is PERMISSION_DENIED; invalid input/proof/password is
INVALID_ARGUMENT; duplicate identity/member/handle is ALREADY_EXISTS; invalid
transitions/processed invitation/owner leave are FAILED_PRECONDITION; hidden
targets are NOT_FOUND. Missing mail is FAILED_PRECONDITION; saturated password
work is RESOURCE_EXHAUSTED. Infrastructure failures retain sanitized failure
status and never imply rollback or authorize mutation replay. Missing/unknown
reason metadata remains a generic transport failure for clients.

## Application operations

The service prepares identifiers/credentials outside repository transactions.
Bcrypt owns a bounded `try_acquire_owned` permit inside its blocking task, so
caller cancellation does not make still-running work free capacity. Expensive
password transitions sample production acceptance time after CPU work. Complete
repository operations still revalidate proof, credential and authority state.

Signup links last 7 days; reset links 24 hours; email challenges 10 minutes.
Sessions default to 180-day validity, 10-minute rotation and 5-minute previous
token overlap with an optional absolute cap. These windows use server time.
Reset consumes its grant and revokes all sessions without issuing a replacement.

Mail configuration is checked before mail-only mutation admission. The maintained
Lettre SMTP transport uses explicit sender/host, mandatory STARTTLS by default
(port 587) or implicit certificate-verified TLS (port 465), and an optional paired
username/password. An explicit nonzero port can override the default.
Plaintext is restricted to an explicit numeric-loopback unauthenticated relay;
no remote downgrade or credential transmission over plaintext is supported.
The former SES SDK, variables and AWS credential chain are removed.
The adapter submits once with 5-second SMTP command timeouts and a 15-second
whole-send budget, without a pool/reconnect retry. CI capture mailers are injected
only into tests. Production has no logged-code fallback.
Delivery occurs after commit; failure preserves state and reports uncertainty.
Without mail, invitations remain pending without automatic signup grants.

Infrastructure reports retain source errors and captured backtraces. RPC logging
prints safe category/uncertainty/backtrace, never raw dependency messages,
request DTOs or authorization metadata. Transport errors contain static sanitized
messages rather than SQL, stack traces or submitted secrets.

## Foreground lifecycle and health

Local default listeners are loopback plaintext HTTP/2 on 50051/50052. Explicit
deployment binds require a trusted internal network and a separate operator TLS
edge for external access. The executable does not issue certificates or provide
native gRPC TLS.

Selected backend roles and optional bundled web children are one failure unit.
Readiness follows configuration, read-only
DB/schema/key checks, dependency construction and successful binds. Partial bind
failure drops already acquired sockets. SIGINT/SIGTERM withdraws admission before
drain. Owned tasks are joined; a component exit or cleanup timeout returns failure.
Defaults are 60 seconds for startup, 20 for graceful shutdown and a further
5 for forced cleanup. Started blocking work is not assumed abortable.

Unary gRPC health for an empty/selected service rereads read-only database
readiness with bounded probe capacity. `chapchoo.liveness` reports process
responsiveness independently. Unknown services return NOT_FOUND; Watch is not
implemented. Optional `--web general|admin|both` starts executable-relative bundled
Node children; `--role none` permits web-only execution without loading DB/key
configuration. There is no daemon, automatic restart or global process scan.
See [Linux delivery](account-delivery.md) for child ownership and package layout.

## Verification boundary

Rust unit/integration suites cover credential admission/cancellation, optional
mail and committed failure, and real two-listener authorization/transport calls.
The root interoperability gate launches disposable CLI instances for both SQLite
and PostgreSQL and calls them using generated Node clients. CLI tests cover
explicit preparation, readiness failure, local recovery, common Auth and owned
termination/partial-startup cleanup. SMTP tests exercise a local protocol fixture,
single-attempt success/failure, required STARTTLS without downgrade, and rejection
of an untrusted TLS certificate, not an external mailbox.

The backend tests do not establish packaged browser or cluster acceptance.
Those verification boundaries are recorded in the
[web](account-web.md) and [delivery](account-delivery.md) Specs.
