---
title: "Agent instructions, default project and configuration refresh"
---

# Agent instructions, default project and configuration refresh

The individual Agent owner manages one saved system-prompt setting. Selected
product, function, owner and execution-only originals remain distinct from that
setting and from the native engine's internal prompt. Saving is not evidence of
native adoption or model compliance.

## Saved owner content

`Agents.GetSystemPrompt` and `SaveSystemPrompt` enforce the same current private
owner and Team membership authority as other Agent operations. Save directly
overwrites the setting; the last committed save wins. There is no stale-edit
condition, setting version, conflict workflow or restore-history interface.
Empty text clears only owner-authored content. An absent setting reads as empty;
a failed read does not. Owner content is limited to 16 KiB of UTF-8 bytes and
rejects NUL. Product/function sources cannot be edited through this setting.

The shared Agent-detail row opens a generous centered desktop modal or mobile
full-screen editor. Save is the only submission action. Close/back discards the
local draft; a pending save owns its submission and disables dismissal. A real
failure retains the draft. Confirmed save closes the editor and invalidates the
saved-value query without claiming that preparation completed.

## Latest selected configuration

A preparation retains exact ordered source-role/text values, the deterministic
rendered document and the session-visible tool catalogue. Source collections are
bounded to 32 entries, the rendered document to 32 KiB, and the combined original,
rendered and catalogue wire material to 48 KiB. No truncation or current-setting
substitution repairs a failed selection.

Each new actual preparation admission reserves its operation as the conversation's
current preparation in the same authority-revalidated transaction. Reserved but
unsealed means pending, not permission to start. First seal checks that exact
current operation before inserting its exact payload. Existing recovery and late
completion never move the pointer backward. The identity is not a setting
revision or a foreign key to a consumable input row.

Component instruction reads require the exact current operation, Connector route,
execution and sealed source. The exact Ready report adds its actual native target
and application time to that operation's evidence. Final input forwarding checks
the current operation's applied evidence, exact execution/source and current saved
owner bytes before committing possible forwarding. Pending, failed or null current
selection cannot borrow an older Ready bundle. Existing delivery facts recover
before this gate and never grant another effect. A rejected transaction can
reobserve preparation before its first grant; possible delivery cannot retry.

Unsealed preparation can be closed as BeforeNativeLaunch only under the exact
current route/operation when a transaction proves no sealed evidence exists.
A lost seal response with persisted evidence does not satisfy that fence.
SQLite and PostgreSQL preserve existing settings, conversations and source data.
Old null causal references require a legitimate new admission; migration does not
infer one from timestamps or operation sorting.

The latest payload survives successful input consumption independently of temporary
preparation/delivery rows. Each new preparation admission removes the Agent's
previous payload in the same transaction that reserves the current operation.
An Agent-unique index limits persistent prompt payloads to one per Agent.
Pending unsealed preparation has no payload; late reports cannot recreate
superseded content. The saved owner setting remains independently overwrite-only.

The owner reads only the current Agent conversation's causally selected payload
through `Conversations.SystemPrompt`. There is no historical-operation read,
history cursor or prompt-content archive. The read-only System prompt page is
server-rendered at the Agent chat's `/system-prompt` route and distinguishes
not-yet-applied content from applied content. Navigation and browser Back preserve
the Agent and Team context. Absent content differs from a route read failure.
Migration 26 deletes all payloads except the current conversation's exact selected
operation, without guessing from timestamps. Downgrade does not recreate deleted
content.
Operational credentials, endpoint bearers and authorization headers are not
instruction sources.

## Default project and publication

The existing persistent native workspace is the general-purpose default project;
there is no second project root or project-management product. Codex maps the
selected document to root `AGENTS.md` and uses standard native instruction loading.
Source role/text originals remain separate from deterministic section headings.

Before taking managed ownership, publication preserves an existing ordinary
`AGENTS.md` once outside instruction discovery. Exclusive archival creation and
verified reentry protect that original. Private no-follow descriptors, finite
blocking ownership, file/directory sync and atomic replacement protect managed
publication. Redirected, special or hard-linked targets fail instead of changing
their referents. Other workspace files are untouched. File success proves disk
bytes, not model adoption. Same-UID hostile mutation is not an isolation guarantee.

## Common refresh and continuing input

The product reconciler checks current prompt content and tool declarations without
waiting for another message. An unchanged configuration reuses the current native
owner. A changed selection uses the same preparation lifecycle for either source.
Server input acceptance stays available while native readiness waits. The existing
consumer retains accepted input within its finite observation budget and forwards
it once only after the selected configuration is Ready. No user Apply/restart,
old-prompt permission or automatic replay is added.
The runtime-owned connection-startup reconciler alone admits and dispatches
startup/project refresh through its reconciliation authority, recovery and drain.
The input processor observes current startup and selected evidence rather than
dispatching refresh itself. Input-owned Setup remains restricted to that input's
existing preparation fact. Queued reads, sealing and forwarding resolve the
current bound submitting session within each Store operation, so token rotation
does not lose an otherwise valid accepted input.

The native owner waits for standard ACPv2 live work evidence, outstanding response
requests, input result acknowledgement and started external tools. Every native
tool call stays owned until actual completion. A terminal command requires a
numeric physical exit; a synthetic failed call or null exit cannot prove it
stopped. Root Idle alone is insufficient. Standard child work requires a live
Running followed by a successful end-turn; idle snapshots, cancelled/error or
disconnected state do not establish completion. Unknown/unbounded observations
cannot become an empty inventory through a timeout.

Root captured originals remain a single-native-session journal. Root standard
child associations are captured unchanged; associated child updates feed only
execution-local work observations at the same serialized ingress. They are not
rewritten as root records. Child permission/form interactions are explicitly
unsupported before root occurrence registration. Unknown lineage or lost child
observations invalidate completeness. An arbitrary restored context has no empty
work proof; only genuine New/pre-work or exact safe local retirement supplies it.

After safe completion, explicit fallible retirement finishes the old physical
owner and its tools. Publication and new actual-start authentication/catalogue
provision precede exact same-context Resume. Initial native title persistence
also makes the unused context restorable. Incomplete cleanup, failed restore or
unknown work reports preparation failure while preserving original context,
source and entered input; it does not force interruption to make refresh succeed.
A later input does not silently use superseded owner instructions.

## Connections

- [Agent profiles](agent-profile.md) — Saved presentation is separate from instructions.
- [Private ownership](agent-ownership.md) — Every mutation and evidence read retains current authority.
- [Conversation storage](conversation-storage.md) — Preparation, consumed input and once-only forwarding.
- [Native lifetime](native-harness.md) — Exact context, physical effects and fallible retirement.
- [Server tools](server-tools.md) — Actual-start catalogue and external-effect ownership.
- [Conversation web](conversation-web.md) — Continuous submission and independent native controls.

## References

- [`system_prompts.rs`](../../../../rust/crates/persistence/src/conversations/system_prompts.rs)
- [`project_refresh.rs`](../../../../rust/crates/persistence/src/conversations/project_refresh.rs)
- [`setup.rs`](../../../../rust/crates/harness/src/runtime/setup.rs)
- [`activity.rs`](../../../../rust/crates/harness/src/runtime/activity.rs)
- [`project.rs`](../../../../rust/crates/harness/src/project.rs)
- [`conversations.proto`](../../../../proto/chapchoo/conversations.proto)
- [Latest-only correction](../../requirements/prompt-261116-latest-system-prompt.md)
