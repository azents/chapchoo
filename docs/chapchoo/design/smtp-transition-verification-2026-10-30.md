---
title: "SMTP Transition Verification"
created: 2026-10-30
tags: [mail, smtp, security, verification]
document_role: supporting
document_type: supporting-verification
---

# SMTP Transition Verification

## Requested outcome

Replace optional SES-specific email delivery with generic SMTP and remove the
AWS SDK dependency graph. Preserve the service mail abstraction, optional mail
admission, manual link flows, post-commit delivery uncertainty and no automatic
resubmission. This change does not send external mail or provision a relay.

## Runtime contract

- `CHAPCHOO_SMTP_SENDER` and `CHAPCHOO_SMTP_HOST` are both required when any SMTP
  setting is present. No SMTP variables means no mail adapter.
- `CHAPCHOO_SMTP_TLS=starttls` is the default: require STARTTLS before sending
  mail or credentials, with default port 587. `implicit` starts TLS immediately
  and defaults to 465. Certificates and hostnames are verified using Lettre's
  Rustls/WebPKI-root transport. No trust-disable or opportunistic TLS setting is
  exposed.
- Optional `CHAPCHOO_SMTP_PORT` accepts only 1–65535. Username/password must be
  supplied together through `CHAPCHOO_SMTP_USERNAME` / `CHAPCHOO_SMTP_PASSWORD`.
- Explicit `loopback-plaintext` is available only for an unauthenticated numeric
  loopback relay (default port 25). DNS names, remote addresses and authentication
  are rejected in that mode. The Helm chart allows secure modes only.
- One SMTP submission with per-command 5-second and total 15-second timeout.
  Connection pooling/reconnect features are not enabled, and there is no retry
  loop. A missing final SMTP reply does not permit resubmission.
- Mail failure Display/Debug remains redacted; configuration and adapter Debug
  do not reveal credentials, sender or body. Backend configuration does not enter
  child web environments.
- Former SES variables, AWS credential chain and Helm fields are removed rather
  than retained as aliases. Deployments must configure a normal SMTP relay;
  an SES SMTP endpoint is also supported as SMTP, without the AWS SDK.

## Dependency and chart boundaries

`lettre` uses explicit `builder`, `smtp-transport`, `tokio1-rustls-tls` features
with defaults disabled. The AWS SDK/SSO/OIDC/STS graph and aws-lc native crypto
are absent from Cargo's resolved workspace graph. Ring remains for TLS; SMTP is
not a promise of zero native cryptography compilation. Test-only rcgen and
Tokio-Rustls build an in-memory fixture certificate; no private key or credential
file is stored.

Helm sender/host/port/TLS values and optional existing Secret username/password
references belong only to backend containers, not web or preparation jobs. The
chart creates no mail credential Secret. Disabled SMTP retains manual link
capability through the unchanged service policy.

## Executed checks

- `cargo test --locked -p chapchoo-mail`: five tests pass, including SMTP envelope
  and body success, explicit refusal and lost final reply without a second
  connection, STARTTLS downgrade refusal, authenticated implicit TLS with a
  trusted test-only certificate, untrusted certificate rejection, and plaintext
  restrictions.
- `cargo test --locked -p chapchoo smtp_`: optional/partial configuration, ports,
  secure defaults, credential pairing and redacted configuration checks pass.
- `cargo clippy --locked -p chapchoo-mail -p chapchoo --all-targets -- -D warnings`
  passes; Cargo format passes.
- Offline chart contracts: 13 tests pass, including backend-only credentials,
  unauthenticated TLS relay and rejection of plaintext/invalid port settings.
- No external SMTP/mailbox, cluster Secret, deployment or provider resource was
  accessed. Integrated full-CI and independent final review belong to the root
  delivery flow; these focused checks do not claim external-mail delivery.
