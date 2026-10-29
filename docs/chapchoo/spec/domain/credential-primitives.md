---
title: "Credential Primitives"
created: 2026-10-29
tags: [accounts, rust, security]
spec_type: domain
domain: credentials
code_paths:
  - rust/crates/chapchoo-domain/src/email.rs
  - rust/crates/chapchoo-domain/src/password.rs
  - rust/crates/chapchoo-domain/src/role.rs
  - rust/crates/chapchoo-crypto/src/lib.rs
  - rust/crates/chapchoo-crypto/src/root.rs
  - rust/crates/chapchoo-crypto/src/code.rs
  - rust/crates/chapchoo-crypto/src/password.rs
  - rust/crates/chapchoo-crypto/src/token.rs
last_verified_at: 2026-10-29
spec_version: 1
---

# Credential Primitives

## Current Reachability

The Rust workspace exposes library primitives with unit tests. There is no
running account server, database persistence, web login or CLI service yet.
These APIs do not establish session expiry, authorize an actor or change a
configured installation key.

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
- The installation-key check is a purpose-separated MAC of a fixed context.
  The primitive checks equality; the future repository owns one-time binding
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

Hashing is synchronous CPU work. The eventual service caller must provide bounded
blocking admission and retain capacity until the work actually completes.
Library primitives have no task spawner, logging or persistent attempt counter.
Unexpected random/bcrypt failures preserve their source while public Display
stays sanitized. Application boundaries own structured reporting/backtraces.

## Verification

`bash scripts/check-rust.sh` runs formatting, warning-denying Clippy, unit and
compile-fail tests, and explicit Cargo dependency-boundary checks. Tests cover
password length/classes/ASCII/full-value verification, fresh salts, root mismatch,
tampered ciphertext/nonce/context/hash, code identity binding and secret-safe
formatting. Required database/network/browser checks remain later-phase work.
