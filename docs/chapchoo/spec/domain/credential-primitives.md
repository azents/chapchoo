---
title: "Credential primitives"
---

# Credential primitives

Reusable domain and crypto values protect representation and secrecy; service and persistence owners supply admission, expiry and installation binding.

# Credential Primitives

## Current Reachability

The Rust workspace exposes library primitives consumed by the account service,
persistence and foreground composition. These primitives do not themselves
establish session expiry, authorize an actor or change a configured installation
key; those contracts belong to the linked service and storage pages.

## Domain Values

Email parsing validates an address without a display-name wrapper, trims outer
whitespace, preserves delivery spelling and uses whole-address lowercasing for
lookup. Dots and plus tags are not removed.

Password creation accepts 8-64 printable ASCII characters and any three of
uppercase, lowercase, digits and the approved special alphabet. Spaces count
toward length but are not a special class. Values are never trimmed or
truncated. Password storage uses zeroizing owned buffers and redacted Debug.

Membership stores Manager or Member. Effective Owner is computed from the
separate Team owner reference; no stored Owner variant exists.

## Common Cryptographic Capability

`RootKey` validates a URL-safe base64 32-byte operator root. `CredentialCipher`
derives purpose-separated internal encryption, email-code and installation-check
keys using HKDF-SHA-256. It exposes no root replacement or rotation API.

- High-entropy bearer values have 32 random bytes, unpadded URL-safe encoding,
  SHA-256 matching records and constant-time digest comparisons.
- Current-token recovery uses XChaCha20Poly1305 with a fresh random 24-byte nonce
  and full tag. Associated data binds purpose, length-delimited session ID and
  current-token digest. Recovery verifies both the authenticated envelope and
  the recovered value's digest.
- Email codes contain six uppercase ASCII letters/digits. Generation uses
  rejection sampling, and submitted ASCII alphanumeric codes normalize to
  uppercase. HMAC-SHA-256 binds canonical email, challenge bearer and code.
- The secret-key check is a purpose-separated MAC of a fixed context.
  The primitive checks equality; the repository owns one-time binding
  and prevents replacing the stored check.

All secret-bearing structures have redacted Debug and explicit access methods,
not implicit Display/serialization. Raw passwords, decoded roots and derived
keys use zeroizing owners. This does not prove that every library/OS copy is
erased. Decryption failures yield no plaintext.

## Password Hashing and Failure Boundaries

`PasswordHasher` uses bcrypt non-truncating APIs, fresh random salts and cost 12
by default. Explicit operational costs must be within 10-31. It verifies the
complete accepted value and rejects oversized login input without truncation.
Encoded verifiers are validated before construction and formatted with redaction.
Login input is checked for the accepted 8-64 printable-ASCII representation
before bcrypt verification, preventing repeated-key equivalence involving
embedded NULs. Creation-only character-class checks are not repeated at login.

Hashing is synchronous CPU work. The service caller provides bounded
blocking admission and retains capacity until the work actually completes.
Library primitives have no task spawner, logging or persistent attempt counter.
Unexpected random/bcrypt failures preserve their source while public Display
stays sanitized. Application boundaries own structured reporting/backtraces.

## Verification

`bash scripts/check-rust.sh` runs formatting, warning-denying Clippy, unit and
compile-fail tests, and explicit Cargo dependency-boundary checks. Tests cover
password length/classes/ASCII/full-value verification, fresh salts, root mismatch,
tampered ciphertext/nonce/context/hash, code identity binding and secret-safe
formatting. Database, RPC and browser evidence belongs to their respective owning pages;
primitive tests alone do not establish those contracts.

## Connections

- [Account service operations and mail](account-operations.md) — Application operations prepare credentials, bound password work and submit optional mail outside database transactions.
- [Account schema and explicit preparation](account-storage.md) — The Store owns durable account identities, schema preparation and installation key binding. Completed account operations are described separately.
- [Account and Team transaction contracts](account-transactions.md) — Account, session, membership and invitation state changes complete under current database authority before external delivery.
- [Execution-bound Connector credentials](connector-credentials.md) — A Connector session belongs to one execution. Bootstrap recovery, operational refresh and revocation have different permissions and retained results.

## References

- [`rust/crates/domain/src/email.rs`](../../../../rust/crates/domain/src/email.rs)
- [`rust/crates/domain/src/password.rs`](../../../../rust/crates/domain/src/password.rs)
- [`rust/crates/crypto/src/root.rs`](../../../../rust/crates/crypto/src/root.rs)
- [`rust/crates/crypto/src/token.rs`](../../../../rust/crates/crypto/src/token.rs)
- [`rust/crates/crypto/src/password.rs`](../../../../rust/crates/crypto/src/password.rs)
