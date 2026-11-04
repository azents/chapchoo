---
title: "Execution-bound Connector credentials"
---

# Execution-bound Connector credentials

A Connector session belongs to one execution. Bootstrap recovery, operational refresh and revocation have different permissions and retained results.

## Execution-Bound Connector Credentials

The assigned currently authenticated Provider can issue/recover the unique session
for a Computer's current, nonretired execution while terminal deletion has not
been accepted. The immutable binding fingerprint is checked and the original
accepted issuance Request ID is returned. Retries with newly prepared server-side
session IDs/keys recover that same binding and bootstrap result; they do not
allocate another session or invalidate operational keys. Revoked bindings cannot
be silently replaced. Bootstrap and operational recovery use distinct domain-
qualified encryption contexts, separate from Provider/account sessions.

Bootstrap exchange authenticates the reusable initial key and current execution.
A genuinely new exchange clears all previous operational-key authority and its
route immediately. Exact still-current request retry only recovers its original
result. Ordinary refresh authenticates the current or immediately previous key;
only the current key may create a successor. The previous key may recover the
exact current refresh response within its original fixed overlap. Request
fingerprints, scope and latest-result identity reject stale exchanges, changed
purpose and attempts to create another successor from a previous key.

Every operation locks/rechecks the Computer/session binding, current execution,
retirement, terminal intent and revocation. Connector exchange/refresh/identity
does not require a live Provider stream or a valid Provider communication key,
and does not acquire a global Provider lock. Provider-authorized issuance and
explicit revocation acquire Provider then Computer authority. Revocation discards
both bootstrap and operational recovery and withdraws only that session's route;
old-session revocation cannot clear another session's connection.

Identity methods return admission snapshots, not reusable permission to accept
future frames. Later messages/reports/heartbeats must recheck current credential
and exact connection authority. Reconnecting with a valid key remains the same
session; replacing an execution requires another authorized binding.

## Connections

- [Computer schema and native identities](computer-storage.md) — Durable Computer state records authority, exact resource identity and cleanup obligations; schema records do not themselves perform external effects.
- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Connector process lifecycle](connector-process.md) — The execution-local Connector exchanges bootstrap authority, maintains its own stream and reports connectivity, not substrate power or harness readiness.
- [Execution reservation, evidence and cleanup](execution-evidence.md) — Reservations precede effects; current-source observations qualify power, retirement and resource absence without granting ambiguous replay.

## References

- [`rust/crates/persistence/src/connector_keys.rs`](../../../../rust/crates/persistence/src/connector_keys.rs)
- [`rust/crates/crypto/src/token.rs`](../../../../rust/crates/crypto/src/token.rs)
- [`rust/crates/service/src/computers.rs`](../../../../rust/crates/service/src/computers.rs)
