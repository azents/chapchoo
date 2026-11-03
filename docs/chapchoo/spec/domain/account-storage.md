---
title: "Account schema and explicit preparation"
---

# Account schema and explicit preparation

The Store owns durable account identities, schema preparation and installation key binding. Completed account operations are described separately.

## Current Reachability

The persistence library exposes explicit preparation/readiness and completed
account/team/session operations with owned asynchronous pool leases. The
[RPC and operator composition](account-rpc.md) calls these library boundaries.

Preparation embeds separate SQLite/PostgreSQL migrations. PostgreSQL stores
native Team-kind, membership-role, grant-kind and invitation-status enums;
SQLite uses equivalent constrained text. Instants are signed Unix epoch
milliseconds in BIGINT columns, supplied by the application acceptance boundary.

## Keys and Relationships

| Table                    | Identity and constraints                                                                                                                 | References and deletion behavior                                                                                                                                           |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| installation_key_binding | Singleton 1; 32-byte derived key check                                                                                                   | No root key stored; persistent across restart/migration                                                                                                                    |
| users                    | Stable text ID; unique canonical email key; protected password hash; boolean server-admin assignment; complete language/time-zone/region | Retained account identity; no deletion capability implemented                                                                                                              |
| display_defaults         | Singleton 1; complete language/time-zone/region                                                                                          | Installation fallback and account-creation defaults, distinct from personal settings                                                                                       |
| teams                    | Stable ID; unique live-Team handle; one owner; unique personal Team per owner; nullable collaborative-only deletion request time         | Owner references users; composite same-Team owner membership is deferred NO ACTION; Agent references block physical deletion until cleanup                                 |
| memberships              | Composite Team/User primary key; Manager/Member only                                                                                     | Team deletion cascades memberships; User deletion is constrained                                                                                                           |
| personal_contexts        | One association per User, unique Team                                                                                                    | Composite deferred membership reference; no automatic collaborative deletion of personal context                                                                           |
| auth_sessions            | Stable ID; unique current hash, indexed previous hash; fixed-size recovery nonce/ciphertext; validity/rotation/revocation facts          | References User; User deletion constrained                                                                                                                                 |
| grants                   | Stable ID; unique hash; signup or password reset; issued/expiry/consumption/revocation facts                                             | Manual/invitation issuer references User; configured-mail signup can have no issuer; reset requires issuer and target User, signup has no target User; deletes constrained |
| invitations              | Stable ID; Team, canonical recipient email; processing state/timestamp consistency                                                       | Team deletion cascades invitations; inviter User deletion constrained; recipient may be unregistered                                                                       |
| login_challenges         | Stable ID; unique binding hash, protected code verifier and expiry/consumption                                                           | Email-bound, no required recipient User; no stored raw code                                                                                                                |
| initial_setup            | Singleton 1 with consumption timestamp                                                                                                   | Initial migration creates availability metadata only, not an administrator                                                                                                 |

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

## Connections

- [Account and Team transaction contracts](account-transactions.md) — Account, session, membership and invitation state changes complete under current database authority before external delivery.
- [Credential primitives](credential-primitives.md) — Reusable domain and crypto values protect representation and secrecy; service and persistence owners supply admission, expiry and installation binding.
- [Computer schema and native identities](computer-storage.md) — Durable Computer state records authority, exact resource identity and cleanup obligations; schema records do not themselves perform external effects.
- [Display language, time zone and regional format](display-settings.md) — Personal values, visitor choices and installation defaults are distinct authorities for the three independent display dimensions.

## References

- [`rust/crates/persistence/src/preparation.rs`](../../../../rust/crates/persistence/src/preparation.rs)
- [`rust/crates/persistence/src/store.rs`](../../../../rust/crates/persistence/src/store.rs)
- [`rust/crates/persistence/migrations`](../../../../rust/crates/persistence/migrations)
