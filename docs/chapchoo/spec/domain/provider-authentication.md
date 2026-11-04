---
title: "Provider bootstrap authentication and scoped token exchange"
---

# Provider bootstrap authentication and scoped token exchange

Provider registration context, canonical owner, execution substrate and evidence
method are independent. Authentication establishes one Provider's authority;
operational keys do not authenticate a new owner or renew themselves.

## Distributed and Standalone admission

`ProviderAuthentication.Exchange` receives a Provider configuration, stable
Request ID and one explicit SA/protected/random-token proof. The configured
adapter verifies evidence before the complete storage operation. Public
component TCP never accepts Trust through an omitted proof, local address or
caller-supplied mode flag.

`serve --standalone-docker` owns a Provider-bound memory authentication endpoint
inside the server process. Its default Trust exchanges through the same scoped
key path. Internal composition, not wire input, supplies the Provider identity;
requesting another ID rejects. Explicit bootstrap policy can prohibit this Trust.
An internal channel is not an OS sandbox: code inside the server process is
trusted, while external Providers do not inherit Trust by sharing the host.

Missing bootstrap Provider metadata is created atomically with first key issuance.
Existing DB-owned metadata is not reset during authentication. Bootstrap cannot
adopt USER/TEAM-owned records or substitute an accepted native kind/daemon.
Existing SYSTEM owner labels and Provider IDs remain stable.

## Server bootstrap policy

`CHAPCHOO_PROVIDER_BOOTSTRAP_CONFIG` is a typed, unknown-field-rejecting JSON
configuration with `serviceAccounts`, `secretKeyProviderIds` and
`trustProviderIds`. These are bootstrap trust permissions, not Admin-editable
ordinary settings. Empty protected-ID permission means that method admits no ID;
there is no separate enabling boolean. The Helm chart supplies coherent default
SA wiring after explicit Kubernetes opt-in.

Authentication method is explicitly selected by distributed Provider
`CHAPCHOO_PROVIDER_AUTH_METHOD` (`service-account`, `secret-key`, `random-token`)
and `CHAPCHOO_PROVIDER_AUTH_FILE`. The absolute regular file is read again for
each exchange, is bounded to 16 KiB, and can follow Kubernetes projected-token
symlinks. The credential itself is not passed as a CLI argument or printed.

## Kubernetes ServiceAccount

The server selects its configured local in-cluster verifier, never a verifier URL
from unverified claims. It submits TokenReview with the expected audience and
requires authenticated status, the returned matching audience and exact
`system:serviceaccount:<namespace>:<serviceaccount>` username.

The namespace/SA name is the allowed identity within the trusted cluster;
audience says what the token is intended for. Historical SA UID is not pinned by
Chapchoo. Kubernetes still validates token/object UID and deletion state, so a
fresh same-name SA can authenticate while a deleted object's old token cannot
be restored by that name alone. Namespace deployment authority to use/recreate
the permitted SA is in the trusted administrative boundary.

Review is bounded external IO outside database transactions. No review success
cache bypasses a fresh exchange. Invalid, unavailable and forbidden verification
remain distinct failures and grant no replacement credential. The Provider's
native API token and projected Chapchoo-auth audience token have distinct uses.
Only Control workloads receive `tokenreviews.create`, not execution Pods or the
Provider SA. External-cluster federation discovery is not an implemented product;
external Kubernetes execution can select another permitted evidence method.

## Protected bootstrap proof

Only the server/issuer holds `SECRET_KEY`. A purpose-specific HKDF-derived AEAD
capability protects the Provider ID plus random salt. The Provider receives an
opaque proof; the server verifies integrity, matching Provider identity and its
bootstrap allowed-ID list. A different installation key or purpose does not
validate. Salt is not replay prevention: a stolen proof is a reusable secret for
that identity while it remains permitted. Reissuing for the same ID does not
invalidate an older valid proof. Remove the ID to prevent further exchange.
No root-key rotation or per-proof revocation registry is introduced.

`provider-credential` is a server-side issuance command using the existing root
only, without database initialization. It writes a private exclusive output file
or creates/validates one immutable Kubernetes credential Secret. Valid existing
material is reused; invalid root/ID/purpose, symlinked private output or conflicting
material fails without overwrite. A create race validates the winner. Secret
output has keep/prune-delete protection and no issuer Job ownerReference.

Helm automates this explicit issuance operation only when an enabled Kubernetes
Provider selects protected authentication. The default SA route creates no such
Secret/Job. The issuer's finite, trusted server-side RBAC includes named Secret
get and namespace Secret create; Kubernetes cannot limit create by resourceName.
The Provider receives only the issued credential, never the root. There is no
unauthenticated remote credential-mint/download service or Docker chart deployment.

## Later-registration capability

Canonical ownership supports SYSTEM/USER/TEAM with exclusive owner fields and
Team foreign keys. Later registered credentials are independently random per
Provider, not root/ID-derived proofs. Their digest and current root-protected
material reside in `provider_registration_credentials`, separate from operational
`provider_keys`. Encryption context binds purpose, Provider and canonical owner.

Exchange checks the current registration digest/revocation and native binding
under the owning transaction. Team/User owner locking precedes the Provider lock;
a deleting Team cannot regain authority through a stale precheck. Team deletion
revokes TEAM Provider registration/operational authority and routes. Referenced
Providers and the Team tombstone remain while Computer/resource cleanup requires
them; only unreferenced Provider rows are collected before Team removal.

This is scoped persistence/authentication/exchange capability, not complete
server/user/team onboarding screens or token-management workflows. Existing
user-owned key management continues to require the current account session;
installation Admin or Team membership does not automatically grant every
Provider's management authority.

## Operational keys, retries and withdrawal

Provider operational keys last 15 minutes; proactive reauthentication starts five
minutes after issuance. Connector keys retain their separate existing policy.
Provider keys cannot obtain a new key by presenting only themselves. The runtime
reuses its selected method and current file; failures do not extend original
expiry or fall back to another method.

Exchange receipt fingerprints bind canonical identity, authentication source and
native configuration, not the raw rotating JWT. Exact current-result recovery
requires fresh authentication and returns the original key/timestamps. Changed
intent conflicts; revoked, superseded or expired results cannot be recovered.
Provider credential mutation IDs use `<created-at-ms>:<opaque-nonce>`. Server
admission rejects a creation time more than one minute ahead and any request at
or beyond 15 minutes from creation, before receipt lookup. Timestamps do not
authenticate a Provider. Retrying within the horizon reuses the exact ID; after
the horizon the caller must begin a fresh authenticated exchange.

Successful credential mutations remove that Provider's expired receipts in the
same transaction, using a Provider/expiry index. Failed mutations roll back
cleanup. No periodic sweeper runs: inactive Providers retain their last history
until the next successful exchange or Provider deletion's existing cascade.
This bounds history duration during activity, not the number of authenticated
requests in a time window. Operational key expiry is independent of retry expiry.

Only the immediately previous operational key has at most five minutes of
overlap, capped by original expiry. Explicit revoke clears overlap/recovery and
withdraws the Provider route.

After a bootstrap-policy withdrawal reaches every Control, an issued key has at
most 15 minutes of remaining authority. TokenReview invalidating an SA token does
not itself erase an already-issued Chapchoo token. Native effects retain their
existing owner and uncertainty; transport/key expiry neither proves cancellation
nor grants replay permission.

## Schema and transition boundary

Forward migration 8 removes obsolete opaque-ID receipts and adds explicit retry
expiry/indexing. It preserves live Provider keys and all canonical/Connector
records. Old opaque callers fail closed and must update; schema rollback cannot
restore removed receipt history. The bounded-retry contract is defined by
[retention-261104](../../design/retention-261104-provider-exchanges.md).

Forward migration 7 preserves Provider/Computer/native identities and Connector
material while revoking old Provider current/previous keys, clearing encrypted
recovery and removing Provider routes. It adds canonical TEAM ownership and
registration credential storage in both SQLite and PostgreSQL. Applied history
is not rewritten; populated reverse migration is rejected instead of erasing
new owner state.

There is no old-owner-bearer fallback, generic `computerProvider` chart alias or
24-hour old-key acceptance window. Coordinated compatible server/Provider
configuration is required across this breaking boundary. Preserving Connector
keys does not imply an old TCP connection survives Control restart; it reconnects
under its original authority. Schema rollback cannot roll back external effects.

## Connections

- [Provider process](provider-process.md) — selected evidence files, renewal and retained native tasks.
- [Control admission](computer-control-rpc.md) — public/internal boundaries and current-source checks.
- [Provider key lifecycle](computer-lifecycle.md) — overlap, exact recovery and revoke.
- [Computer storage](computer-storage.md) — preserving migrations and canonical native identities.
- [Helm deployment](deployment.md) — explicit prerequisites, conditional issuer and scoped permissions.
- [Agent ownership](agent-ownership.md) — account/team cleanup and private authority.

## References

- [`provider_authentication.rs`](../../../../rust/crates/rpc/src/provider_authentication.rs)
- [`component_keys.rs`](../../../../rust/crates/persistence/src/component_keys.rs)
- [`provider.rs`](../../../../rust/crates/crypto/src/provider.rs)
- [`provider_credential.rs`](../../../../rust/apps/chapchoo/src/provider_credential.rs)
- [`runtime.rs`](../../../../rust/apps/chapchoo/src/runtime.rs)
- [Provider authentication snapshot](../../requirements/provider-261104-authentication.md)
