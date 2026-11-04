---
title: "Account service operations and mail"
---

# Account service operations and mail

Application operations prepare credentials, bound password work and submit optional mail outside database transactions.

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

## Connections

- [Account RPC boundaries](account-rpc.md) — Account transport owns service admission, caller credentials and validated wire outcomes. Business transactions and process lifetime have separate owners.
- [Account and Team transaction contracts](account-transactions.md) — Account, session, membership and invitation state changes complete under current database authority before external delivery.
- [Credential primitives](credential-primitives.md) — Reusable domain and crypto values protect representation and secrecy; service and persistence owners supply admission, expiry and installation binding.

## References

- [`rust/crates/service/src/service.rs`](../../../../rust/crates/service/src/service.rs)
- [`rust/crates/mail/src/lib.rs`](../../../../rust/crates/mail/src/lib.rs)
- [`rust/crates/crypto/src/password.rs`](../../../../rust/crates/crypto/src/password.rs)
