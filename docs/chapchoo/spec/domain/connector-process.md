---
title: "Connector process lifecycle"
---

# Connector process lifecycle

The execution-local Connector exchanges bootstrap authority, maintains its own stream and reports connectivity, not substrate power or harness readiness.

## Computer Connector process

`chapchoo computer-connector` requires an explicit trusted-network Control
endpoint, execution session ID, explicit saved root and bootstrap key. The initial key is read only
from `CHAPCHOO_CONNECTOR_INITIAL_KEY`, not a CLI argument. The process has no
product DB, installation root, Provider or owner credential dependency.

It exchanges the bootstrap key, opens its own operational stream, verifies Hello
acceptance, sends heartbeats, requires finite acknowledgements and uses current
command-dequeue checks before reporting execution connectivity. It handles only
ObserveConnector; readiness here is connectivity, not harness or power state.
Proactive key refresh uses an absolute schedule retained across reconnects; a
refreshed key starts a new authenticated stream before the old overlap ends.
Disconnection creates a fresh bounded-admission attempt after a one-second delay,
never replay of previous calls. Revoked/unauthorized authority ends the process.
SIGINT/SIGTERM cancels its owned loop under a finite shutdown budget.

One [native owner](native-harness.md) is composed outside the reconnect/key-refresh
loop and explicitly finished when that loop exits. Constructing this idle owner
does not prepare or start a harness. Public conversation command routing is not
activated by this foundation; ObserveConnector still reports connectivity only.

A real child-process test provides only execution bootstrap inputs, waits for the
current Connector route, revokes the Computer Provider key, then obtains an
independent Connector observation and verifies owned process shutdown.

## Connections

- [Execution-bound Connector credentials](connector-credentials.md) — A Connector session belongs to one execution. Bootstrap recovery, operational refresh and revocation have different permissions and retained results.
- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Provider process and allocation acknowledgements](provider-process.md) — External trusted Providers own native IO, while authenticated server operations own durable mutations and evidence acknowledgements.
- [Native preparation and lifetime](native-harness.md) — Saved storage, managed/existing supply, strict ACP v2 and verified private-scope cleanup.

## References

- [`rust/apps/chapchoo/src/connector.rs`](../../../../rust/apps/chapchoo/src/connector.rs)
- [`rust/apps/chapchoo/tests/connector_cli.rs`](../../../../rust/apps/chapchoo/tests/connector_cli.rs)
