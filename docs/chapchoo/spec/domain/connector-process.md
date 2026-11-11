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
command-dequeue checks before reporting execution connectivity. Generated command
operations distinguish observation, sealed setup, exact source-bound input,
typed answers and Stop.
The readiness field remains connectivity, not harness or power state.
Proactive key refresh uses an absolute schedule retained across reconnects; a
refreshed key starts a new authenticated stream before the old overlap ends.
Disconnection creates a fresh bounded-admission attempt after a one-second delay,
never replay of previous calls. Revoked/unauthorized authority ends the process.
SIGINT/SIGTERM cancels its owned loop under a finite shutdown budget.

One [native owner](native-harness.md) is composed outside the reconnect/key-refresh
loop and explicitly finished when that loop exits. Constructing this idle owner
does not prepare or start a harness. After the stream is installed, the Client's
[connection-startup reconciler](conversation-storage.md#connection-time-preparation)
can send sealed Setup for the selected real conversation without a message.
It does not await setup inside stream admission. Owner conversation services
route explicit commands separately; ObserveConnector still reports connectivity
only.

Setup dequeue additionally checks its current server-sealed intent, saved supply
and fresh-versus-existing context permission through component RPC. A bounded
execution-local worker admits the native actor command and immediately returns
queue admission to Control. The actor retains setup fingerprints and actual
outcomes across stream changes; duplicate operations recover their original fact
and changed intent conflicts. Missing local facts are not permission to recreate
a context.

Setup source selection reopens the existing complete copy and resumes its exact
native session without history replay. Missing expected source fails. Creating
an initial copy/context requires the sealed positive permission and local
empty-copy checks; restore failure never falls through to New. Setup outcomes
are reported later, separately from Control queue acknowledgement. The local
worker retains bounded result reports across reconnect and uses current
credentials to retry publication, never the native command.

Setup also supplies a narrow authentication provider to the native actor.
Receiving Setup does not itself select/apply a credential: only a real launch,
after preparation, calls that provider. Exact already-live reuse preserves the
existing applied record and file. The provider uses current component authority
to recover/admit the start application, acquire short-lived native material,
assemble source-consistent 32-KiB pieces and check final handoff. It never
extracts an ambient account or sends the server refresh bundle to native.
Initial API-key login and external-token seeding are described by the
[native owner](native-harness.md#actual-start-authentication).

A separate coalesced credential worker runs at most one task and checks every
five seconds while a route is active. Its first local binding read returns
absence for an idle native owner, without contacting credential RPC or starting
native work. For a live owner it acquires only the existing applied application
under the current route/key, skips unchanged source payloads and hands changed
material to the actor's exact-binding file-publication operation. It does not
select a new saved record. Network acquisition is outside the actor; only final
authority checking and serialized publication enter it. Ordinary link,
uncertain-refresh or eligibility failure does not trigger native restart.
Shutdown closes scheduling and cancels network waiters, while server refresh
and local blocking publication retain their respective physical owners.

Input commands carry small operation/source references. The worker pulls the
sealed original text through authenticated 32-KiB byte reads, validates constant
identity/size/offset and complete UTF-8 assembly, then rechecks the exact source
at the final empty-suffix read before native queue admission. Per-input 16-MiB and
aggregate 32-MiB worker budgets are independent of the 32-operation/report limit.
An existing actor result is recovered without redispatch. Missing or uncertain
facts do not become known non-delivery; only actual insertion/steering/failure
results are reported. Shutdown drains workers alongside native retirement.

Answer commands likewise carry small operation/source references. Complete
original typed values are pulled with consistent responder token, captured
occurrence and kind, followed by the final empty-suffix check. Conversion uses
supported ACP SDK response types. The native actor checks its actual responder
before consuming it and reports local responder acceptance or exact rejection;
neither queue admission nor local acceptance proves remote consumption.

An invocation lease covers each operation kind/identity from local admission
through native result, durable report and native-fact release. Concurrent
duplicates cannot occupy another local dispatch owner. After successful durable
publication, the command-checking Connector loop
enqueues release of only the matching known native result. A background report
task cannot release it while an earlier command check is still awaiting its
response. The lease remains held through native release acknowledgement; later
commands are fenced by the server's settled operation checks. An uncertain or
different native result is never released. A refused report does not authorize
native-fact release: a transport status alone is not proof of irreversible source
fencing. This recycles acknowledged capacity without
adding a persistent outgoing journal or expiring captured history.

After captured-source catch-up, the same coalesced worker publishes actual native
foreground/work observations and current responder references. Each state points
to its establishing captured occurrence; each request uses its actual execution
token and captured question. Closed responders are reported explicitly and a late
open report cannot revive their server slot. Full original questions stay in
normalized source storage rather than oversized status frames.

Stop validates its sealed server operation at dequeue and queues the exact
source/session/work target. The native actor publishes cancellation only while
that work identity is still current. A known cancellation publication and an
actual closed-target rejection are reported separately from Control admission.
Neither changes Computer power or ends the native owner.

An independent coalesced capture worker reads the current retained source through
the native owner's narrow read handle. A 500-ms scheduling cadence keeps at most
one synchronization task active across stream replacement. An absent/unbound
source does not start native work or create a copy. The worker discovers current
server conversation state, binds an available source with the exact expected
previous identity when needed, resolves the stored occurrence, and transfers one
finite `CapturePass` through component-only capture RPCs. Raw events use 32-KiB
pieces under the unchanged 64-KiB frame budget. It reports finite completion only
after all events through that boundary are published; later append uses another
pass.

Each server call rechecks current source authority. Old-route/key failure leaves
the retained copy intact; the next current-route pass rereads durable progress.
This repeats observations, never native prompts/answers/Stop, native replay or
upstream gap repair. Blocking pass reads retain their read permit in the actual
closure. Process shutdown stops capture admission and awaits its physical read task
alongside native-owner retirement, so capture transfer does not delay native stop.

A real child-process test provides only execution bootstrap inputs, waits for the
current Connector route, revokes the Computer Provider key, then obtains an
independent Connector observation and verifies owned process shutdown.
It also loads a synthetic saved source larger than one wire frame through the
actual child Connector, then verifies raw UTF-8 bytes and finite server completion.
This model-free source transfer does not certify actual Codex execution.
The same real-process fixture drives setup through the server input service and
PeerDelivery. Its unprivileged non-PID1 profile rejects native launch; later
authenticated setup failure reaches the Store independently of the short
product acceptance and Control acknowledgement. This is rejection-path evidence,
not authenticated model execution.
The same fixture ingests original form/state observations, then routes a complete
large typed answer and exact-work Stop through their real server processors and
the actual child. With no live native responder/work owner, both produce known
rejection reports while preserving original answer values; owned SIGTERM cleanup
still succeeds. It does not certify successful model-side response consumption
or cancel publication.

## Connections

- [Execution-bound Connector credentials](connector-credentials.md) — A Connector session belongs to one execution. Bootstrap recovery, operational refresh and revocation have different permissions and retained results.
- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Provider process and allocation acknowledgements](provider-process.md) — External trusted Providers own native IO, while authenticated server operations own durable mutations and evidence acknowledgements.
- [Native preparation and lifetime](native-harness.md) — Saved storage, managed/existing supply, strict ACP v2 and verified private-scope cleanup.
- [Conversation storage](conversation-storage.md) — Exact captured source, atomic pair publication and bounded component transfer.

## References

- [`rust/apps/chapchoo/src/connector.rs`](../../../../rust/apps/chapchoo/src/connector.rs)
- [`rust/apps/chapchoo/tests/connector_cli.rs`](../../../../rust/apps/chapchoo/tests/connector_cli.rs)
