---
title: "Codex Text Conversation Architecture Decisions"
created: 2026-11-08
tags: [agent, conversation, codex, acp, architecture]
document_role: primary
document_type: adr
snapshot_id: chat-261108
---

# Codex Text Conversation Architecture Decisions

Requirements: [chat-261108/REQ](../requirements/chat-261108-text-conversation.md).
Mode: Collaborative. The requester owns material technical choices. This record
accepts only the individually selected decisions below; a complete Design and
implementation approval do not yet exist.

## Decision Map

- Fixed product outcomes: initial Codex/ACP v2; ongoing text/Markdown conversation;
  steering; explicit current-work stop; inline forms and permission responses;
  retained reentry/recovery; URL elicitation and product response-timeout excluded.
- Established boundary: the Connector owns its harness connection and observed
  state across an ephemeral server link, and supplies inspection on reconnect.
- Accepted D1: Connector-owned harness lifetime, distinct from server-link lifetime.
- Accepted D2: Chapchoo-defined compatibility constraints with newest eligible
  release selection for managed downloads.
- Accepted D3: check and prepare the managed release before a new harness process
  starts; retain the selected versions throughout that running harness lifetime.
- Accepted D4: after transient update lookup/download failure, allow a complete
  eligible managed cache combination; report failed freshness confirmation.
- Accepted D5: the execution-local Connector owns managed harness preparation;
  Providers retain Computer substrate lifecycle and saved-storage responsibilities.
- Accepted D6: preserve managed artifacts in the same Computer's existing saved
  volume/PVC across execution-environment replacement, subject to reuse admission.
- Accepted D7: acquire from official upstream distribution with mandatory digest
  matching against official HTTPS metadata; do not claim signature verification.
- Accepted D8: existing-installation support supplies the Codex CLI, while the
  ACP adapter and its runtime remain managed by Chapchoo.
- Accepted D9: initially select an existing Codex CLI by an explicit absolute
  path inside the executing Computer; defer automatic discovery conveniences.
- Accepted D10: the private Agent owner's supply selection is authoritative
  server-held product state, delivered to its authorized Connector context.
- Accepted D11: permit saving supply changes while live; retain the running
  configuration and apply saved changes only at the next actual harness start.
- Accepted D12: start the harness on authorized user text submission when needed
  in an available Computer environment; reuse an existing live harness.
- Accepted D13: keep the normal harness after task completion; initial scope
  excludes idle-time automatic retirement.
- Accepted D14: orderly lifetime shutdown uses bounded cooperation/output drain,
  then forced cleanup if needed; uncertain retirement is not success.
- Accepted D15: trust the owner's explicitly selected existing CLI installation,
  checking usability/compatibility without requiring official-artifact equality.
- Accepted D16 direction: server-maintained product records remain separate from
  native model context and current Connector observations. Exact canonical-store
  versus projection, loading and reconciliation mechanisms remain under design.
- Accepted D17: copy incoming ACP session events into local Connector JSONL,
  replay only on first session load by default, and assign missing event IDs.
  Native-side missed-event recovery and external harness activity are excluded.
  A hypothetical reload is not an approved product feature.
- Accepted D18: snapshot/event cursors drive Connector-to-server captured-event
  replay; missing snapshots fail explicitly and require server rebuild. Store
  raw and service-normalized event data as pairs. Exact product-fact removal
  boundaries and browser read synchronization remain unresolved.
- Accepted D19: unavailable-source-snapshot rebuild completely replaces the
  session's server conversation data, including independently admitted product
  facts. Preserve no separate old-input/answer/delivery-evidence exception.
  Browser read synchronization and rebuild publication remain under design.
- Accepted D20: load recent normalized records, page older history, apply live
  normalized updates and re-read the current conversation region on browser
  reconnection. No separate durable browser change feed in the initial design.
- Accepted D21: retain input and return short server product acceptance with an
  operation reference; separately report preparation, native acceptance and
  execution/result evidence. Dispatch-journal and recovery contracts remain open.
- Accepted D22: no separate persistent Connector dispatch journal. Use retained
  server records/events and surviving Connector command state; unproven outcomes
  after Connector replacement remain uncertain and are not automatically replayed.
- Accepted D23: local recording-failure input gating/controlled shutdown is
  limited to a small reuse of existing paths; omit specialized handling if it
  needs substantial extra work. No dedicated storage-management subsystem.
- Accepted D24: the server owns ChatGPT refresh and coordinates renewal of the
  saved authentication; Computers receive required short-lived token material.
  Native application remains conditional; no adapter fork or single-consumer
  restriction is selected.
- Accepted D25: save changed authentication selection without stopping/restarting
  a live harness; retain its applied authentication and use the saved selection
  at the next actual start. Underlying authority loss and same-record credential
  replacement remain separate contracts.
- Accepted D26: automatically replace the native credential file on same-record
  key replacement/reauthentication; do not terminate, restart, re-login or force
  reload of the harness. File publication does not prove live token adoption.
- Accepted D27: keep native credential files in the Computer's existing saved
  volume/PVC, without an additional ephemeral mount; restrict file access but
  acknowledge persistent plaintext residue and same-identity access limits.
- Accepted D28: deliver execution credentials through the existing explicitly
  trusted-network HTTP component boundary; no new public endpoint, relay or
  encryption protocol, and no transport-confidentiality claim.
- Accepted D29: retain the complete current captured snapshot until actual
  source replacement or authorized storage deletion. Server ACK does not prune
  its prefix; no automatic rotation/compression/expiry subsystem is added.
- Accepted D30: clear old session conversation data before refill; use ordinary
  conversation loading/error UI until replacement is complete, not an internal
  reconstruction label or a second staged dataset for atomic swap.
- Confirmed product scope: initial existing Codex installation support under
  chat-261108/REQ-14, alongside the download-based cloud preparation direction.
- Remaining design work is consolidated into six work packages: artifact/cache
  preparation; execution environment and retirement; product records/native
  context; command/event delivery and recovery; credential application; complete
  Design authority, replacement and verification.
- Discuss the six work packages in order, one package at a time. Within each,
  resolve consequence-level decisions without separate interviews for equivalent
  helper, field or directory choices. Cross-package research does not combine
  their decision authority.
- Unselected material contracts include product-record/native-context authority;
  long-running command, event catch-up and uncertainty; execution trust,
  credential application/refresh and live authority withdrawal.
- Concrete supported artifact/runtime profiles, supply defaults, cache validation
  and capacity, launch environment and whole-tree retirement still require
  grounded specification and feasibility evidence. Existing accepted decisions
  remain constraints; consolidating work does not approve new mechanisms.
- Agent-owned local details: equivalent helper decomposition, identifiers, file
  layout, and fixture composition that introduce no new state or contract.

## D1. Couple Harness Lifetime to Its Owning Connector

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, explicitly confirmed after comparing coupled lifetime
  with a persistent external supervisor/broker.
- Applies to: REQ-1, REQ-5, REQ-10, REQ-11, REQ-12.

### Context

The user-visible conversation must survive ordinary UI reentry and Computer-off
record viewing. A native Codex session contains model context; a current execution
is live work in that context. These are distinct from a Computer execution and
its Connector authority. Context restoration is not proof of resuming an
interrupted command at its previous instruction.

The server link may disconnect without ending the Connector or its harness. That
ordinary reconnect is different from failure of the Connector process itself.
The current Connector implementation has execution-scoped authentication and
observation only; the native harness lifecycle is new work, not an already
implemented shutdown guarantee.

### Decision

The Connector and its managed ACP harness belong to one owned execution lifetime.

- Loss or replacement of the server communication link alone does not terminate
  the managed harness. The surviving Connector retains the native connection and
  observed state, then exposes current inspection after reconnect.
- Failure/termination of the owning Connector must retire its managed harness
  rather than deliberately retain that harness for adoption by a successor.
  A parent/child PID relationship alone is not sufficient termination evidence.
  The complete Design must establish and verify actual supervision, shutdown,
  process-tree cleanup and uncertain-cleanup boundaries.
- A successor must not overlap an unretired predecessor execution merely because
  its server stream or parent process disappeared. An unconfirmed predecessor
  remains uncertain until the applicable ownership/retirement evidence is known.
- Reentry keeps the same user conversation. A replacement harness restores the
  original native session's saved context where supported and verified. It does
  not silently substitute fresh context if restoration fails.
- Interrupted commands/work are not automatically replayed or claimed to resume
  at the exact prior point. The user sees retained evidence and truthful failure
  or uncertainty and can provide the next instruction through the approved
  recovery experience.
- This does not undo external effects already produced by the previous work.

### Considered Alternative

A persistent supervisor/broker could retain the harness's stdio and work while
Connectors restart and reconnect to that broker. It would improve live execution
survival across Connector replacement, but requires another lifetime owner,
authenticated handover, fencing, event retention and pending-request transfer.
A new process cannot simply recover the predecessor's anonymous stdio handles.
That additional live-process adoption contract is not selected for this snapshot.
Neither alternative preserves live process execution after the Computer execution
environment itself is lost.

### Consequences and Remaining Validation

- User conversation, native context and live process/execution lifetimes remain
  separate concepts. Exact identity/storage mechanisms are not selected here.
- Durable product records, native context location and restoration, synchronization
  guarantees, credential handling and protocol adapters are subsequent decisions.
- Protocol v2 session resume synchronizes context/current state but is not a
  process-adoption guarantee. Codex ACP 2.5.0 includes resume/history/state paths;
  actual Rust/Codex restoration and managed child cleanup still require evidence.
- Operating-system-specific supervision must be justified in the Design without
  making a successful transport close equivalent to physical termination.
- No runtime implementation, SDK adoption, credential delivery, rollout or
  deployment is authorized by accepting this decision alone.

## Supply and Maintenance Direction — Detailed Decisions Pending

On 2026-11-08 the requester kept existing harness installations open as a supply
source, favored download-based preparation for cloud Computers, and requested an
update policy. The preparation/maintenance scope is reflected in revision 2 of
chat-261108/REQ before resolving these mechanisms.

This direction does not yet select implicit PATH discovery, fallback between
supply sources, a particular installer or download owner, upstream latest on
launch, automatic update application, or replacement of a running harness.
Initial Codex/ACP v2 scope and accepted D1 remain unchanged. Executable provenance,
compatibility validation, and ownership of the launched process are separate
contracts even when the executable was installed outside Chapchoo.

The next discussion must separate:

- Which approved combination may be used: the ACP adapter, native Codex and any
  required runtime, rather than treating one executable's presence as readiness.
- Who prepares and maintains downloaded artifacts, and who owns changes to an
  externally installed executable.
- When an update is discovered, prepared and applied, including running work,
  saved-context compatibility, failures and recovery.

No additional numbered ADR decision is accepted by this direction note.

## D2. Select the Newest Release Within Chapchoo Compatibility Constraints

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: requester-proposed compatibility constraints with newest
  eligible release selection, explicitly confirmed after reviewing the trade-offs.
- Applies to: the preparation/maintenance scope in chat-261108/REQ revision 2 and
  execution readiness and continuity in REQ-1, REQ-11 and REQ-12.

### Context

Pinning one complete harness combination to a Chapchoo release makes adoption of
each subsequent harness release depend on a Chapchoo release. Selecting upstream
latest without Chapchoo compatibility constraints allows changes beyond the
integration's declared support boundary. An independently curated exact-version
catalogue would introduce a separate release-management contract.

The requester proposed that Chapchoo define compatibility constraints and permit
the newest release inside those constraints. The inspected Codex ACP 2.5.0 source
itself declares a native Codex dependency range, so independently selecting the
newest adapter and native executable is not evidence of a compatible combination.

### Decision

Chapchoo defines the supported harness compatibility constraints. Managed
acquisition selects the newest eligible release satisfying those constraints,
rather than one permanently fixed exact release or unconstrained upstream latest.

- New releases inside the declared constraints are eligible without a Chapchoo
  patch release for each exact version. Changing the declared constraints remains
  a Chapchoo change.
- Eligibility must account for the target platform and the combined adapter,
  native Codex and required runtime constraints. Independent version selections
  must not violate those dependency requirements.
- A release satisfying a declared range is eligible under a support policy; it
  has not necessarily been individually tested by Chapchoo. This explicitly
  trades some exact-version predictability for faster adoption within the range.

### Consequences and Remaining Decisions

- This selects version-resolution policy, not when to check, download or apply an
  update. In particular, replacement or restart of a running harness is not
  authorized by this decision.
- Exact range bounds, prerelease/channel eligibility, actual acquisition source,
  metadata and artifact verification, resolver/preparation ownership, cache
  concurrency, offline and failed-update behavior remain unresolved.
- The Design must distinguish the declared constraints from the actual selected
  artifact versions. Exact recording and storage mechanisms are not selected.
- Compatibility with saved native context must still be established; satisfying
  package constraints alone is not evidence of successful context restoration.
- The existing-installation supply direction remains viable. Discovery, trust,
  compatibility admission and ownership of changes to external installations
  require their own decisions; D2 does not authorize overwriting those files.
- The initial Codex/ACP v2 scope, D1 lifetime boundary and prohibition on uncertain
  work replay remain unchanged.

## D3. Check Before a New Harness Start and Keep Running Versions Stable

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, check at a new harness process start rather than only
  when preparing the Computer execution environment.
- Applies to: the preparation/maintenance scope in chat-261108/REQ revision 2,
  REQ-1, REQ-11 and REQ-12; builds on D1 and D2.

### Context

A Computer execution environment can outlive an individual harness process.
Resolving its managed release only when preparing that environment can leave
later harness starts on an older selection for the entire Computer lifetime.
Resolving at each new harness start allows subsequent starts to adopt eligible
updates, at the cost of remote lookup and potentially download work on startup.

A user message, a model invocation and a new managed harness process have
different lifetimes. Updating on a conversation turn would disrupt an otherwise
surviving harness and is not necessary to adopt updates at new process starts.

### Decision

Before starting a new managed harness process, check the newest eligible release
under D2 and prepare the required artifacts. Retain the selected artifact
versions for that running harness lifetime.

- Do not stop, restart or replace a running harness merely to apply an update.
- Sending a user message, reaching another model turn, reopening the UI or
  reconnecting the surviving Connector's server link is not a new harness start
  and does not trigger version replacement.
- When a genuinely new harness process is needed, perform a new version check
  even if its Computer execution environment has remained available.
- This policy remains subject to D1 retirement and non-overlap obligations and
  the existing context-continuity and non-replay requirements.

### Consequences and Remaining Decisions

- This defines the update check/application boundary, not the trigger for the
  first harness launch, an idle-retirement policy or an automatic restart policy.
- Startup has remote lookup and possible download costs. Lookup/download budgets,
  failure reporting, cache admission and whether a usable cached combination may
  be used after a failed update attempt remain separate decisions.
- Preparation/cache ownership, concurrent starts, artifact publication and
  protection of versions in active use still require a concrete Design.
- D3 applies to managed release acquisition. It does not transfer ownership of
  an externally installed executable to Chapchoo or authorize modifying it.

## D4. Permit Eligible Managed Cache After Transient Update Failure

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, permit an eligible existing managed cache combination
  after a transient lookup/download failure rather than block every new start.
- Applies to: the preparation/maintenance scope in chat-261108/REQ revision 2,
  REQ-1, REQ-11 and REQ-12; builds on D2 and D3.

### Context

D3 puts a version check and possible download on the new-harness startup path.
Requiring successful freshness confirmation on every such start would make a
transient registry, distribution endpoint or network failure prevent execution
even when a usable managed combination is already available locally.

Cache presence alone is not evidence of completeness, current eligibility,
integrity or compatibility with the native context to be restored.

### Decision

After a transient update lookup/download failure, a new managed harness may
start using an existing complete managed cache combination only when that
combination satisfies current compatibility constraints, integrity admission
and the required saved-native-context compatibility conditions.

- Preserve truthful distinction between freshness confirmation failure, the
  actual versions used and successful preparation of a new release. Do not
  report that the latest release was installed or confirmed.
- If no usable cached combination exists, report preparation failure and do not
  start the harness under this policy.
- File presence or partial preparation is insufficient admission. An artifact
  that fails verification must not be executed.
- This permission covers transient lookup/download availability failure, not
  permission to ignore integrity or compatibility failures, use out-of-range
  versions, or perform an arbitrary downgrade for saved-context restoration.
- Do not silently switch to an external installation or another supply source.
  D1 retirement and context continuity and the existing prohibition on replaying
  uncertain work remain enforced.

### Consequences and Remaining Decisions

- Startup can remain available during a transient distribution outage without
  claiming that the selected cache is the latest available upstream release.
- Precise failure classification and budgets, cache validation mechanisms,
  preparation/publication ownership, storage and concurrent admission still
  require subsequent ADR/Design work and feasibility evidence.
- D4 does not authorize offline model execution, bypass authentication or network
  prerequisites, or establish successful native session restoration merely from
  a cache check.

## D5. Let the Connector Own Managed Harness Preparation

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, preparation inside the Computer under the Connector
  rather than a Provider-side artifact preparation and delivery layer.
- Applies to: the preparation/maintenance scope in chat-261108/REQ revision 2,
  REQ-1, REQ-11 and REQ-12; builds on D1 through D4.

### Context

The Connector already runs inside the Computer execution environment, whereas
Providers own native Computer allocation, retirement and saved storage. D3 needs
preparation at each new managed harness start, including starts after the Computer
environment has already been prepared.

A Provider-side preparation layer could centralize access to artifact sources,
but would need new on-demand preparation requests, artifact delivery and completion
evidence during a running Computer execution. Computer-boot-only installation
would not satisfy D3.

### Decision

The execution-local Connector owns orchestration and admission of managed harness
preparation before launch.

- It coordinates version selection, download, verification and preparation,
  applying the accepted D2 through D4 policies.
- It remains responsible for complete prepared-artifact admission when
  preparation fails or multiple start requests overlap. Partial installation is
  not a runnable preparation result.
- Providers retain Computer creation, retirement and saved-storage management;
  this snapshot does not add Provider-side harness installation/delivery RPCs.
- Preparation operates within the Computer's non-root, read-only-rootfs security
  baseline and an explicitly authorized writable area. Selecting this owner does
  not authorize root execution or making the root filesystem writable.

### Consequences and Remaining Decisions

- The Computer needs access to the selected distribution sources for managed
  acquisition. Those sources and their trust/integrity contracts are still
  pending, not established by selecting this owner.
- The Connector gains installer/cache coordination responsibility. Concrete
  concurrency, publication, validation and cancellation mechanisms still require
  Design and feasibility evidence; preparation success is not harness readiness.
- Cache persistence, scope, storage location, capacity and cleanup remain
  undecided. D5 does not select an additional volume or a directory in saved data.
- Existing-installation discovery, admission and update ownership remain separate;
  D5 does not authorize modifying externally installed files.
- D1 lifetime and retirement obligations remain unchanged. Preparation ownership
  does not introduce a persistent broker that retains live harness work across
  Connector replacement.

## D6. Preserve Managed Cache in the Computer's Existing Saved Storage

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, retain managed artifacts in the Computer's existing
  saved volume/PVC rather than only for the current execution environment.
- Applies to: the preparation/maintenance scope in chat-261108/REQ revision 2,
  REQ-1, REQ-11 and REQ-12; builds on D2 through D5.

### Context

Both current Computer adapters preserve a Computer-specific saved environment
independently from its execution resource. The Connector runs as non-root with a
read-only root filesystem and an existing writable saved-volume mount.

Execution-only caching would require an additional temporary writable area and
would lose reusable managed artifacts when that execution environment is
replaced. Retaining them can reduce repeated downloads and extend D4's availability
benefit to subsequent Computer executions.

### Decision

Store and preserve the managed harness artifact cache in the same Computer's
existing saved volume/PVC so it can be reused across Connector replacement and
Computer execution-environment replacement.

- Use the existing Computer-specific saved storage; this cache decision does not
  introduce a separate persistent volume or a cross-Computer shared cache.
- New harness starts still perform the D3 update check. A preserved cache is not
  permission to skip that check or claim freshness.
- Preserved artifacts remain subject to current eligibility, integrity and
  required saved-context compatibility admission under D4 before reuse.
- Cache cleanup must remain distinct from deletion of conversation/native-context
  data and credentials. Choosing cache persistence does not choose their storage
  locations or retention policies.
- A directory boundary alone is not security isolation from processes able to
  modify it under the same identity. Persistence is not a trust assertion.

### Consequences and Remaining Decisions

- Managed cache uses saved-storage capacity and needs bounded retention and
  cleanup rules, including incomplete preparations and artifacts in active use.
  Those rules and validation mechanisms remain pending.
- The exact internal directory layout is an agent-owned implementation detail
  only while it preserves these storage, ownership and deletion boundaries.
- This does not change the existing original-volume/PVC identity and lifecycle
  contract, promise reconstruction after original storage loss, or authorize
  recreating or replacing an owner's missing saved storage.

## D7. Trust Official Upstream Metadata and Require Artifact Digest Matching

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, official upstream distribution plus mandatory digest
  verification, rather than additionally requiring publisher signature/provenance
  verification for every eligible managed release.
- Applies to: the preparation/maintenance scope in chat-261108/REQ revision 2,
  REQ-1, REQ-11 and REQ-12; builds on D2 through D6.

### Context

Official npm metadata for the inspected adapter, native Codex launcher package
and Linux native package declares exact versions, artifact URLs and SHA-512
integrity. It also declares registry signature and provenance metadata. Official
Node distribution metadata provides SHA-256 archive digests. Metadata presence
does not establish that any artifact, signature or provenance has been verified.

Requiring publisher signature/provenance verification would add trust-root,
expected-issuer/source, key-rotation and evidence-availability contracts. The
requester selected official-distribution trust with mandatory artifact digest
matching as the initial baseline.

### Decision

Managed acquisition uses official upstream distribution and treats metadata
obtained from the official HTTPS endpoint as the expected-artifact authority.
Bind exact version, target platform, artifact identity and expected digest, then
compare the actual downloaded archive bytes with that digest.

- Expected digest absence or mismatch prevents installation and execution of
  that new artifact. There is no proceed-without-verification exception.
- The policy applies to the managed adapter, native Codex and required runtime
  artifacts, not just the first downloaded file.
- Initial managed acquisition does not support arbitrary download URLs or custom
  mirrors. This does not select a particular package-manager invocation,
  distribution format or exact runtime version.
- Report digest verification as such. Do not imply that publisher signatures or
  provenance were cryptographically verified under this baseline.

### Consequences and Remaining Decisions

- This policy trusts the official distribution service. If its metadata and
  artifact are both maliciously changed, digest matching alone cannot detect that
  compromise. D7 does not claim independent publisher/build-origin attestation.
- Cache admission must remain consistent with this trust baseline and D4/D6.
  Retaining expected digests, validating extracted files and handling same-identity
  writers require concrete mechanisms and feasibility evidence; a mutable
  adjacent digest file alone is not independent tamper-proof authority.
- The selected source endpoints/formats, safe extraction, preparation publication,
  package lifecycle-script handling and cache validation mechanisms remain pending.
- A digest failure is not reclassified as D4's transient availability failure.
  D7 adds no permission to bypass verification or switch supply sources.
- External-installation selection and trust remain separate. D7 does not label
  an arbitrary installed executable as official or verified.

## D8. Combine an Existing Codex CLI with a Managed ACP Adapter

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, reuse an existing Codex CLI while managing the ACP
  adapter and its runtime, rather than accept a fully external ACP launch stack.
- Applies to: chat-261108/REQ-14 and REQ-1; builds on D1 through D7.

### Context

REQ-14 confirms initial support for an explicitly selected existing Codex
installation in the executing Computer. A native Codex CLI and an ACP adapter
are separate components. Reusing the former does not require outsourcing the
ACP-facing component or accepting an arbitrary external adapter/runtime stack.

The inspected Codex ACP source supports starting an explicitly supplied Codex
path with app-server arguments. This is static interoperability evidence, not a
successful Chapchoo Computer launch test.

### Decision

For the initial existing-installation path, use the selected existing Codex CLI
and a Chapchoo-managed ACP adapter and adapter runtime.

- Managed adapter/runtime preparation follows D2 through D7. The selected CLI
  must be compatible with the managed adapter and the integration's current
  support conditions.
- The selected existing CLI is externally maintained. Do not replace it with
  the latest managed Codex or overwrite/update it as a preparation side effect.
- Initial support does not accept an externally installed ACP adapter/runtime
  stack as an alternative ACP-facing execution configuration.
- Required execution dependencies of the existing CLI still need validation.
  Neither an executable path nor its mere presence proves a runnable combination.
- Using an existing CLI does not adopt ambient credentials or change D1 process
  lifetime, current authorization or selected-authentication boundaries.

### Consequences and Remaining Decisions

- Chapchoo retains control of the ACP-facing adapter compatibility boundary while
  allowing reuse of an existing Codex installation.
- Executable selection/discovery, configuration authority and location, external
  CLI provenance/trust, version admission and handling of external file changes
  remain pending. D7 does not attest an arbitrary external CLI's origin.
- Concrete distribution formats and runtime versions, launch arguments and
  environment, session restoration and shutdown still require Design and actual
  feasibility evidence.
- This composition is not permission to silently switch supply sources after
  missing-file, incompatibility or startup failure.

## D9. Initially Select the Existing CLI by an Explicit Absolute Path

- State: Accepted by the requester on 2026-11-08 as the initial approach.
- Selected alternative: A, an explicit absolute path in the Computer, with
  discovery/selection conveniences deferred for later refinement.
- Applies to: chat-261108/REQ-14 and REQ-1; builds on D8.

### Context

PATH-based selection is convenient for conventional CLI installations, but
environment order or installation changes may select a different executable.
An explicit absolute path makes the intended installation clear without adding
automatic discovery or a general shell-command configuration surface.

### Decision

Initial existing-installation support selects the Codex CLI using an explicit
absolute executable path interpreted inside the executing Computer.

- Inspect and use that selected path rather than searching PATH or known install
  locations to substitute another CLI.
- A missing or ineligible selected executable is an error for that selection,
  not permission to choose another installation or download managed Codex.
- Treat the path as an executable selection, not an arbitrary shell command
  string. Concrete argument/environment contracts remain subsequent work.
- Automatic discovery and richer selection conveniences are deferred. They are
  not prerequisites for the initial integration or authorized by this decision.

### Consequences and Remaining Decisions

- Setup requires knowledge of the intended executable path. Its configuration
  authority, scope and storage location remain pending; D9 adds no implicit
  installation-Admin, Team-manager or Agent-owner mutation authority.
- An absolute path is not an immutable file identity. Symlinks and external
  updates can change its target or bytes. Target/version validation, permitted
  CLI wrapper/runtime forms and change handling still need concrete contracts.
- D9 does not attest an external CLI's origin, choose native-context/authentication
  storage, adopt ambient credentials or permit execution outside the Computer.
- The requester explicitly chose this bounded initial approach and deferred
  convenience refinement; no additional path-selection UX is required now.

## D10. Keep Owner Supply Configuration as Server-Authoritative Agent State

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, Agent-owner product settings stored by the server,
  rather than operator-only Connector startup configuration.
- Applies to: chat-261108/REQ-15, REQ-14 and REQ-1; builds on D5, D8 and D9.

### Context

Current cloud creation atomically assigns a private Agent and its dedicated
Computer. Owner reads/mutations require individual ownership and current Team
membership; installation administrator or Team-manager status alone does not
grant private-Agent mutation authority.

REQ-15 confirms a retained owner product selection. Keeping selection only in
deployment/Connector startup settings would require operator changes and would
not provide that approved owner capability.

### Decision

Store the Agent's supply mode and applicable existing-CLI absolute path as
server-authoritative product configuration scoped to that private Agent.

- Admit owner reads/changes under the current account session, private Agent
  ownership and current Team membership boundaries.
- Deliver the admitted configuration only to its applicable authorized
  Agent/Computer/Connector execution context. Existing bootstrap credentials and
  runtime configuration retain their separate authorities.
- The Connector applies the admitted selection when preparing the harness.
  A harness-writable local file is not an alternative authority for mutating
  the saved product selection.
- Providers retain their substrate responsibilities. This does not add a
  private supply-setting mutation capability to installation Admin or Team
  management surfaces.

### Consequences and Remaining Decisions

- The feature needs retained configuration and owner read/update contracts.
  Schema, exact DTO fields, UI placement and equivalent local decomposition are
  subsequent Design work, not independently approved mechanisms here.
- Initial defaults, update admission during live work, application timing and
  any explicit restart/apply-now behavior remain unresolved.
- This selects configuration authority, not command/event delivery, replay,
  reconnect synchronization or credential-application semantics.
- D10 does not authorize arbitrary shell configuration, ambient authentication,
  download URL overrides, a user-facing release selector or Computer sharing.

## D11. Save Supply Changes While Live and Apply Them at the Next Harness Start

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, allow saved changes during live execution and defer
  application, rather than reject changes while a harness is alive.
- Applies to: chat-261108/REQ-15 revision 5, REQ-1, REQ-11 and REQ-12;
  builds on D3 and D10.

### Context

D3 retains selected managed artifact versions throughout a running harness
lifetime. D10 separately introduces owner-controlled supply mode and existing-CLI
path settings. Changing these settings is not the same operation as selecting
a newer managed artifact release, so its live-application behavior needs an
explicit contract.

Rejecting all changes while a harness survives avoids saved-versus-running
differences, but makes configuration depend on first ending that harness.
Task completion or model-idle state is not proof of process termination.

### Decision

An authorized owner may save supply-setting changes while the harness is alive.
Keep the running harness on its existing configuration and apply the saved
selection when a genuinely new harness process starts.

- Saving alone does not start, stop or restart the harness or Computer.
- Preserve the distinction between saved selection and observed running
  configuration. Report a difference when they diverge; save acknowledgement is
  not successful application or harness readiness.
- The next message, model turn or task-completion event is not a new process
  start. A surviving harness can continue using its existing configuration.
- Prepare and validate the changed selection on the applicable new-start path.
  Failure does not authorize silent supply reversion, context reset or replay.
- D1 predecessor-retirement and non-overlap obligations still apply. Saving a
  desired change is not proof that an old execution has ended.

### Consequences and Remaining Decisions

- Saved desired configuration and actual active configuration can differ.
  Their concrete representation, start-time binding and race/correlation
  mechanisms require Design without introducing an unapproved version-history
  product or assuming that transport acknowledgement proves application.
- This does not introduce a separate apply-now/restart control or change existing
  Computer lifecycle authority.
- Initial supply defaults, initial harness launch triggers, external file-change
  admission and readiness/session-restoration contracts remain pending.

## D12. Start the Harness on Demand at User Text Submission

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, start when user text needs an absent harness rather
  than eagerly start at Computer execution-environment preparation.
- Applies to: chat-261108/REQ-1 revision 6, REQ-2, REQ-11 and REQ-12;
  builds on D1, D3 and D11.

### Context

Eager harness startup can move preparation/initialization latency before first
input, but also starts processes and authentication preparation for Agents that
have not been used. On-demand startup keeps that work on an actual text-send
path, at the cost of possible first-send preparation delay.

The current Connector implements connectivity/observation, not harness startup.
Selecting either trigger adds a new harness lifecycle; neither is current
implemented behavior.

### Decision

When an authorized user submits text in an available Computer execution
environment, reuse the existing live harness or prepare/start one if no live
harness is available and the applicable safe-start conditions are satisfied.

- Do not launch a new harness per message when a live one already exists.
- UI navigation/reentry and record reads alone do not start a harness.
- Computer-environment or Connector availability alone is not an eager-start
  trigger. This decision does not introduce automatic Computer power-on.
- New starts apply D3 artifact checking and D11 saved supply selection, together
  with D1 predecessor-retirement/non-overlap obligations.
- This does not replay interrupted or uncertain previous work. After failure,
  unknown predecessor state still cannot be overwritten by a new start merely
  because another message arrives.

### Consequences and Remaining Decisions

- First input can wait for managed preparation, process launch, ACP initialization,
  selected-authentication application and native session setup/restoration.
  Input retention and truthful readiness/delivery remain required.
- ACP initialization and active native session setup are distinct from process
  presence; session-scoped prompt admission requires the active session under
  ACP v2. Exact state projection and authentication mechanisms remain pending.
- This selects a startup trigger, not an idle-retirement timer, automatic crash
  restart, Computer power policy or message queue/acceptance implementation.
- Launch environment, exact preparation/distribution mechanisms, native context
  binding, ready/error states and shutdown evidence still need Design and
  feasibility validation.

## D13. Retain the Harness After Normal Task Completion Without an Idle Timer

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, retain a normally running harness rather than add
  safe-idle detection and automatic idle-time retirement initially.
- Applies to: chat-261108/REQ-1 revision 7, REQ-11 and REQ-12;
  builds on D1, D3, D11 and D12.

### Context

Task/turn completion does not imply process termination or context deletion.
Retiring an idle harness can reduce retained process resources, but introduces
idle-admission evidence, new-input/retirement races and repeated preparation and
native-context restoration. Time without visible output is not proof of safe
idle when work or user requests may still be pending.

### Decision

After normal task completion, retain the existing harness for subsequent text
and context reuse. Do not introduce idle-time automatic harness shutdown in the
initial integration.

- Task completion alone does not close the native session or retire its process.
- A later message reuses the live harness under D12. Retention is not permission
  to start a new task without user input or replay prior work.
- Retain the running artifact/configuration selection under D3/D11 while that
  harness survives. Task completion does not apply pending supply changes.
- Connector/Computer termination, authority withdrawal and process failure remain
  distinct lifecycle events governed by their applicable contracts.

### Consequences and Remaining Decisions

- Normally idle harness processes retain resources until an applicable separate
  lifecycle event. Idle resource reclamation is deferred, not required now.
- The policy does not turn current-work Stop into disabling all future work or
  provide a new harness power control.
- It does not authorize autonomous crash restart, live adoption by a successor
  Connector, or continued execution after authority withdrawal.
- Shutdown cooperation/escalation, whole-tree retirement evidence, launch
  environment and native-context binding remain pending.

## D14. Use Bounded Cooperative Shutdown Before Forced Cleanup

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, bounded cooperative shutdown followed by forced
  cleanup when necessary, rather than immediate force for every orderly shutdown.
- Applies to: REQ-1, REQ-11 and REQ-12; builds on D1 and D13.

### Context

The managed harness lifetime must end when its owning lifecycle requires it.
Immediate force can shorten shutdown but lose undelivered output/state and
native-context flush opportunities. Unlimited cooperation cannot establish a
bounded retirement contract.

Current-work Stop, managed harness lifetime shutdown and Computer retirement
remain separate operations. Ordinary server-link loss alone does not retire a
surviving Connector's harness.

### Decision

For an orderly harness-lifetime shutdown, stop admitting new work, use supported
cooperative cancellation/termination and allow bounded output/status drain.
Escalate to forced cleanup when the bounded opportunity is exhausted.

- Verify actual retirement of the owned process scope. A signal, closed pipe,
  expired waiter or direct-child exit alone is not proof of whole-tree cleanup.
- Unconfirmed cleanup remains failure/uncertainty, not successful retirement.
  Do not overlap a successor with that unretired/uncertain predecessor under D1.
- Crash, uncatchable termination and owner death require an independent cleanup
  path; they cannot rely on executing graceful-shutdown code.
- Preserve already-retained evidence and truthful partial outcomes. This does
  not promise rollback of external effects or guaranteed final output delivery.

### Consequences and Remaining Work

- Exact supervision, process-scope enforcement, retirement evidence and budgets
  must be established against the supported Computer environments.
- Existing Connector/Docker time budgets and adapter shutdown code are evidence
  constraints, not approval of new grace numbers or proof of native cleanup.
- Shutdown budgets are distinct from product response deadlines or permission
  expiry, which remain excluded.
- D14 does not add automatic Computer restart, a new power control, autonomous
  work replay or live harness adoption after Connector replacement.

## D15. Trust Owner-Selected Existing CLI Installations

- State: Accepted by the requester on 2026-11-08.
- Selected alternative: A, trust the explicitly selected existing installation
  while validating usability and compatibility, rather than admit only existing
  installations whose bytes can be matched to official distribution artifacts.
- Applies to: REQ-14 and REQ-15; builds on D7 through D10.

### Context

An explicit executable path identifies the intended installation, but neither
that path nor a version response establishes publisher provenance. Requiring
official-artifact equality would narrow existing-installation support, including
wrapper and repackaged installation forms that cannot satisfy that comparison.

### Decision

Treat the private owner's explicitly selected existing Codex installation as a
trusted execution input, subject to execution usability and compatibility checks.
Official-artifact byte equality is not an initial admission requirement for that
external installation.

- Keep this trust path distinct from managed acquisition's D7 official-source
  digest verification. Do not label an external installation publisher-verified
  merely because it exists, reports a version or is compatible.
- The selected CLI may execute with the admitted backend authentication under
  the later credential-application contract. Usability/compatibility checks are
  not a guarantee against a malicious or modified executable.
- Continue to enforce private ownership, current Team admission, the explicit
  Computer-local path and no silent supply substitution. Existing CLI files are
  not automatically overwritten or updated.
- This does not grant ambient-login adoption, relax the Computer security
  baseline or claim protection against arbitrary same-identity code.

### Consequences and Remaining Work

- Concrete supported CLI wrapper/runtime forms, start-time target/version checks
  and secret-safe diagnostics require implementation specification and validation.
- Artifact cache admission and runtime feasibility still follow D2 through D7.
  D15 does not weaken managed-download integrity or add a new supply source.
- This closes the existing-CLI provenance policy within the supply/cache topic;
  it is not complete Design approval or evidence of a successful native launch.

## D16. Keep Server Product History Distinct From Native Context and Live State

- State: Direction accepted by the requester on 2026-11-08, with an explicit
  request to resolve canonical/projection semantics and complete session-data flow.
- Selected alternative: A, server-maintained normalized product records rather
  than making the native protocol log the product-history authority.
- Applies to: REQ-2, REQ-3, REQ-6, REQ-8 through REQ-12.

### Decision Boundary

The server retains the product conversation record independently of whether its
Computer is available. Native Codex context remains responsible for supported
model-context continuation. Current Connector observations, not mere saved
history, establish the evidence for present execution state.

- Product history, native context and last-observed/current execution state must
  not be treated as interchangeable sources.
- Native message identities and supported content updates must be preserved;
  normalization does not authorize invented messages or silent context resets.
- Server retention does not by itself prove native input insertion, model
  consumption, completeness of native history or successful context restoration.

### Explicitly Unresolved by This Acceptance

The requester asked to design, rather than assume, whether normalized domain
records are themselves canonical or a projection of another server-held source.
This direction therefore does not accept event sourcing, an event-log authority,
an internal version-history system or a particular normalized schema.

The same boundary applies to initial/partial/incremental loading, atomic
read/subscription handoff, missed-update detection, native baseline import,
unknown-session admission and replay reconciliation. Any new cursor/change-feed,
baseline publication or version-related behavior needs explicit specification
and authority; recording this direction does not silently select it.

## D17. Capture ACP Session Events Locally Without Native Gap Recovery

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: local JSONL event copies, first-load replay and subsequent
  live capture, with native-side gaps outside the recovery scope.
- Applies to: REQ-3, REQ-11 and REQ-12, Requirements revision 8.

### Context

ACP v2 supports no-history resume and full retained-history replay from `start`.
It does not guarantee a stable ID for each event occurrence or a standard partial
replay position. Entity IDs identify update targets, and JSON-RPC request IDs
correlate calls; neither is a generic event-occurrence ID.

The Connector manages the harness lifetime under D1. The requester chose a
bounded capture contract rather than native-gap detection/reconciliation or
coverage of harnesses independently launched outside that managed lifetime.

### Decision

- The Connector stores local JSONL copies of incoming ACP session events.
- By default, request retained-history replay only when first loading that
  session. Subsequent session loads/resumes do not request history replay.
  Opening/reopening the product UI is not a new first native-session load.
- Preserve incoming replay data on that first load and copy subsequent real-time
  session events into JSONL as they arrive.
- Preserve an actual event-occurrence ID when present; otherwise assign one
  while copying the event. An entity/message ID is not a substitute for an
  event-occurrence ID.
- A locally assigned ID identifies a captured event. It cannot retroactively
  identify an uncaptured event, prove upstream completeness or deduplicate
  independently regenerated replay occurrences.
- Ignore native-side missed events: do not add detection, automatic replay,
  repair, timestamp matching or reconciliation for those gaps. Independently
  launched external harness activity is outside this integration's coverage.
- The requester's hypothetical `reload` means replaying from the start and
  replacing that session's previous local event-copy data. It is not yet a
  product feature, UI action or API and is not implemented by this decision.

### Authority and Remaining Boundaries

D17 selects the native-capture portion left open by D16. D16's separation of
server product records, native context and current observations remains in force.
The earlier unaccepted native-gap repair/reconciliation proposal is superseded.

- JSONL is a local captured-event copy, not a selection of server canonical
  storage, event sourcing or a command/dispatch journal.
- Native replay/replacement does not authorize deletion of server-held product
  records, accepted inputs, original questions/answers or captured partial output.
- First-load replay does not authorize adopting arbitrary unknown sessions.
- Missing native events are outside recovery scope, not proof of non-delivery,
  non-execution, current request validity or successful completion.
- This decision does not select fsync/crash-loss guarantees, retention/capacity
  policy, storage-failure shutdown, server delivery/acknowledgement, browser
  change feeds or automatic reconstruction from repeated native replay.
- The Design still needs to specify first-load identity/lifecycle boundaries,
  event decoding/content application and local capture failure handling within
  this scope. Any material unresolved policy returns for a decision.

## D18. Catch Up From Snapshot/Event Cursors and Pair Raw With Normalized Data

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: snapshot-scoped event replay from Connector copies,
  rebuild on unavailable snapshot, and paired server event representations.
- Applies to: REQ-3, REQ-11, REQ-12 and REQ-16, Requirements revision 9.

### Decision

- The Connector assigns a new snapshot ID whenever it performs the full
  replay/replacement reload described in D17.
- The server uses the snapshot ID together with an event-occurrence ID as its
  Connector catch-up cursor.
- A request for an old snapshot that no longer exists fails explicitly with
  snapshot-unavailable semantics. Do not silently reinterpret its event ID
  against a different snapshot.
- On connection, the server supplies its latest retained cursor. The Connector
  replays captured events after that cursor from its local copy.
- If that snapshot is gone, the server rebuilds its data from the available
  snapshot. This is source-snapshot replacement, not native-gap reconciliation.
- The Connector continuously forwards subsequently received harness events
  with their event-occurrence IDs.
- For each stored event, the server retains its service-normalized data paired
  with its original raw-format data. Rendering uses normalized data; debugging
  and re-normalization use raw data.

### Ownership and Scope

Connector replay here reads the D17 captured-event copy. It does not request
another native replay or recover events missed before capture. Raw source is
retained for renewed normalization, while its corresponding normalized data is
the rendering representation; this is no longer a normalized-only server store.
Native model context and present execution authority remain distinct.

The requester explicitly selected snapshot identity and snapshot-scoped cursors
for this boundary. That authorization does not imply per-message versions, a
general history/rollback system or a separate server-to-browser change feed.
It also does not add the hypothetical reload to the product UI or API.

### Remaining Design Work and Material Boundaries

- Define the initial snapshot/cursor when no prior data exists and the concrete
  ordering/lookup contract. Event IDs alone do not imply chronological sorting.
- Specify consistent pair ingestion and publication of server progress. The
  latest retained cursor must not falsely assert persistence of unretained pairs;
  transaction, acknowledgement and duplicate-delivery details remain to be
  designed rather than inferred as a dispatch journal.
- Define the server rebuild boundary: rebuilding ACP-derived data is selected,
  but whether independently admitted user inputs, original form/permission
  answers and delivery evidence are removed is unresolved. These facts may
  have no reconstructible native event.
- Establish rebuild publication/failure handling and avoid treating reconstructed
  historical requests as evidence that an execution/request is currently valid.
- Respect the existing secret/ownership boundaries when retaining raw session
  events. This is not permission to archive credential-bearing authentication
  traffic or expose raw debug data to unauthorized consumers. Raw-content
  retention, debugging access and storage limits still need concrete design.
- Initial page/older-page/browser-live synchronization remains part of topic 3;
  D18 selects the Connector/server cursor, not a browser change-feed mechanism.
- No actual native replay, storage/reconnect or rebuild validation has been run.

## D19. Completely Rebuild Session Conversation Data on Snapshot Replacement

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: B, complete reconstruction rather than retaining
  independently admitted product facts outside the replaced event data.
- Applies to: REQ-8 through REQ-12 and REQ-16, Requirements revision 10.

### Decision

When D18's previous source snapshot is unavailable, rebuild the affected
session's server conversation data completely from the currently available
snapshot.

- The replacement boundary includes previous raw/normalized event pairs,
  rendering state, independently admitted user inputs, original form/permission
  responses and delivery evidence.
- There is no separate preservation exception for server-admitted facts that
  cannot be reconstructed from the new snapshot. Such facts may be lost.
- Rebuild does not merge the old snapshot back into the new one, infer missing
  answers or delivery results, or introduce native-gap repair.
- This is a conversation-data replacement boundary, not deletion of the owning
  Agent, account, saved authentication/supply selection or Computer.
- Reconstructing data does not dispatch historical user input, Stop, form or
  permission effects again. A missing old receipt is not evidence of confirmed
  non-delivery; reconstructed historical requests are not proof of current
  request validity.

### Relationship to Earlier Decisions

D19 resolves D18's open removal boundary and explicitly grants replacement of
server conversation data, including the product facts excluded from automatic
deletion by D17 alone. D16's native-model-context/current-observation separation
remains in force. The requester selected full replacement rather than the
unaccepted proposal to preserve independent product facts.

Within the same retained snapshot, normal catch-up and retained-record behavior
still follow D17/D18 and the Requirements. Neither ordinary UI reentry nor an
ordinary Connector-link reconnect constitutes complete rebuild by itself.

### Remaining Design Work

- Specify rebuild publication, partial/failure handling and reads while
  reconstruction is incomplete. No particular staging/atomic-swap mechanism
  is selected by this decision.
- Keep current execution/request admission truthful across replacement without
  recreating an unapproved separate archive of deleted conversation facts.
- Define initial/older/browser-live reads separately; D19 does not select a
  server-to-browser change feed.
- Complete replacement is accepted policy, not evidence of runtime feasibility
  or successful reconstruction tests.

## D20. Re-read Current Normalized State on Browser Reconnection

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, normalized current-state re-reading rather than a
  separate durable cursor-based browser change feed.
- Applies to: REQ-3, REQ-6, REQ-8 through REQ-13 and REQ-16,
  Requirements revision 11.

### Decision

- On first entry, read recent normalized conversation records from the server.
  Fetch older history incrementally when the user reads farther back.
- While connected, deliver real-time normalized message, activity and request
  updates to the screen.
- On browser reconnection, re-read the currently read conversation region from
  the server's latest normalized state and reconstruct the screen cache.
- Do not add a separate durable browser change feed or require browser replay
  of every missed transport event in the initial design.
- This is a read/cache-reconstruction path, not deletion of server records,
  native replay, new harness startup or historical operation dispatch.
- D18's snapshot/event cursor remains the Connector-to-server captured-event
  catch-up contract. It is not promoted into an all-product browser cursor.

### Consequences and Remaining Work

Reconnection retransmits the currently read region rather than only its missed
deltas. The server's retained records, not browser transport history, supply the
recovered state. D19 still governs complete server conversation-data replacement
when its source snapshot becomes unavailable.

- Define bounded recent/older reads and large-content delivery without arbitrary
  content truncation. Current unary RPC frame limits remain relevant.
- Reuse the existing SSR/client initial-data handoff and current owner/Team
  admission. Raw debug data is not a browser rendering payload.
- Specify initial-read/live-update ordering so an older response cannot erase a
  newer normalized update, including history pagination and Agent navigation.
- Define reconnect/rebuild-in-progress handling within this current-state read
  policy. Do not silently add a persistent version/change-feed system.
- Live transport, publication/failure handling and authorization withdrawal
  still require grounded Design and verification. This decision is not evidence
  that current unary APIs already provide browser subscriptions or no-gap reads.

## D21. Separate Short Product Acceptance From Native Processing Results

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, short server acceptance and separate operation state,
  rather than holding Submit open through preparation and ACP acceptance.
- Applies to: REQ-1, REQ-2, REQ-4, REQ-10 through REQ-12,
  Requirements revision 12.

### Decision

- Validate current owner/Team/target admission and retain the submitted input
  before returning product acceptance and an application operation reference.
- Do not hold that Submit response until harness preparation, ACP acceptance or
  native work completion. Treat those as separately observed states/results.
- Keep the application operation identity distinct from a transient Control
  call/connection identity and from a native message/entity ID.
- The Connector performs needed preparation and native processing under the
  applicable current authority. Product acceptance alone does not extend that
  authority or prove native input insertion, consumption or successful work.
- Report preparation, harness acceptance, execution and result evidence
  separately. Browser connection loss alone does not cancel or resubmit the
  operation; reconnection uses current server operation state under D20.
- Running input remains steering under REQ-4, not a separate queued future task.
  Uncertain native effects are not automatically dispatched again.

### Context and Consequences

Current Control pending calls are connection-local and bounded to short waits.
Its queue time consumes the original deadline; loss after possible forwarding
is uncertain. Extending that waiter to the complete preparation/work lifetime
would not create persistent acceptance or outcome evidence.

The new product operation owner must outlive an individual browser waiter.
Its implementation and result-publication contract remain Design work. This
decision is not a claim that the existing Control already provides that owner.

### Remaining Boundaries

- Persistent Connector dispatch evidence versus lifetime-local command state,
  duplicate admission, reconnect settlement and result correlation still need
  a coherent delivery contract.
- D17's incoming JSONL copy is not approval of an outgoing dispatch journal.
  Acknowledgement, retention/capacity and storage-failure behavior remain open.
- D19's complete conversation-data replacement also applies to these product
  records. Do not add an undisclosed preservation exception for old operations;
  lost evidence still does not authorize replaying an uncertain effect.
- This does not authorize unlimited admission, future-task scheduling,
  credential fallback or changed native steering behavior.

## D22. Recover Commands Without a Separate Persistent Dispatch Journal

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, existing retained evidence and lifetime-local command
  state rather than an additional persistent outgoing dispatch journal.
- Applies to: REQ-2, REQ-4, REQ-10 through REQ-12 and REQ-16,
  Requirements revision 13.

### Decision

- Do not add a separate local persistent journal of pre-effect dispatch facts
  and command results.
- Preserve D21's server product acceptance/state records and D17/D18's incoming
  captured-event copies. This decision does not remove those records.
- A surviving Connector retains bounded command-processing state in its native
  lifetime owner, outside the ephemeral server-stream loop.
- On ordinary stream replacement, recover/report operation state using retained
  server facts, captured events and the surviving Connector's processing facts.
  Reconnecting or looking up results does not dispatch the native command again.
- After Connector failure/replacement, settle only outcomes justified by retained
  evidence or a currently provable correlation. Otherwise retain uncertainty.
  Missing command state is not proof of non-delivery.
- Do not infer correlation from equal text, a similar timestamp or an unrelated
  native entity ID. Uncertain effects are not automatically resubmitted.

### Rationale and Consequences

D1 retires the managed native harness when its Connector is lost and excludes
persistent live-process adoption. The integration already declines automatic
replay of uncertain work. A separate outgoing journal is therefore not required
by the selected recovery scope.

The alternative journal could retain additional dispatch/results across a crash,
but would add persistence, retention, capacity and storage-failure contracts. It
would still leave uncertainty between a native effect and recorded result.

### Remaining Design Boundaries

- Define bounded current-command admission, duplicate prevention and exact
  target/request validity. Evicting temporary state cannot authorize executing
  the same uncertain operation again.
- Define evidence-backed result publication/correlation and incoming pair/cursor
  persistence without introducing the rejected outgoing journal indirectly.
- Local incoming-event retention, capacity and recording-failure policy remain
  unresolved. No unlimited-memory or lossless arbitrary-outage claim is made.
- D19's complete session conversation-data replacement remains applicable.
  Neither a hidden command archive nor uncertain-effect replay is authorized
  when previous server conversation data is replaced.

## D23. Keep Local Recording-Failure Handling Proportionate

- State: Accepted with an explicit implementation-effort limit by the requester
  on 2026-11-09 (KST).
- Selected alternative: controlled failure handling only when it is a small
  reuse of existing error/admission/shutdown paths; otherwise omit the
  specialized behavior.
- Applies to: REQ-2, REQ-5, REQ-11, REQ-12 and REQ-16,
  Requirements revision 14.

### Decision and Effort Limit

The requester initially selected blocking new effect-bearing input/responses and
controlled managed-harness shutdown when local JSONL event recording fails.
The requester then qualified that selection: skip it if it adds substantial work.

- Where the normal JSONL append error can simply reuse the existing planned
  admission/error path and D14 shutdown owner, block new effect-bearing
  submissions/responses and initiate controlled harness retirement.
- Preserve Stop/shutdown control and already-stored records. This is neither
  Computer power-off nor external-effect rollback.
- Do not create dedicated disk monitoring, a storage-recovery framework,
  separate failure-state machinery or a new supervisor to deliver this edge case.
- If the special admission/shutdown coupling itself requires substantial extra
  implementation, omit it rather than expanding the feature for this scenario.
  Keep ordinary error propagation truthful; failed writes are not successful
  persistence and ongoing work cannot be claimed to have stopped without proof.
- The implementation-effort discretion is limited to this specialized local
  recording-failure handling. It is not general authority to remove other
  conversation, lifecycle or authentication requirements.

### Boundaries and Remaining Evidence

Ordinary server-link loss still follows D1/D18: preserve live native work and
local capture. This policy does not introduce network-disconnect shutdown,
automatic native reload/restart, history truncation, lossless crash capture or
an outgoing dispatch journal.

The native owner and shutdown machinery are not implemented yet. Reuse appears
architecturally available under D14, but actual small implementation cost is not
proven. The complete Design/implementation plan must state whether the direct
reuse path fits this limit; it must not present a comprehensive fault-management
system as already approved or necessary.

## D24. Keep ChatGPT Refresh Authority on the Server

- State: Direction accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, centralized refresh authority and short-lived token
  delivery rather than independent native-managed refresh copies.
- Applies to: REQ-1, REQ-10 through REQ-12 and REQ-17,
  Requirements revision 15.

### Decision

- The server owns the protected refresh credential and the renewed saved
  authentication bundle. Coordinate renewal at the authentication-record scope
  rather than allowing each Computer to rotate a copied refresh token.
- Provide only the short-lived token material required for admitted native use
  to the exact authorized Computer/Connector execution. Do not export the
  refresh token as part of native credential-file seeding.
- Preserve use of the same eligible saved authentication by multiple Agents;
  do not add a one-active-consumer restriction to make native refresh simpler.
- Revalidate current saved-selection, authentication scope and execution
  authority for token delivery and renewal-result publication. A stale refresh
  result cannot replace a newer user credential replacement or another account.
- Renewal that needs the server is unavailable while that server path is
  unreachable. This does not by itself terminate existing work on link loss.
- If supported native application cannot meet this direction, report the
  feasibility constraint. Do not silently add an adapter fork, native bridge,
  ambient login fallback or API-key-only reduction.

### Evidence and Feasibility Boundary

Inspected stock ACP supports explicit API-key authentication but exposes no
stored external ChatGPT-token login/refresh bridge. Exact native 0.165.0 source
supports loading `ChatgptAuthTokens` mode from the native authentication schema.
Its auth manager also caches credentials; an external file write alone is not
proof of live adoption. These are source-backed integration candidates, not
proof that the full ACP/native refresh and reload path works.

The selected direction is therefore conditional on native application and
refresh validation. File-schema parsing, generated protocol types or a single
successful login cannot establish renewed-token adoption during live work.

### Remaining Material Boundaries

- Concrete supported token-application path, confidential delivery and native
  credential storage/residue lifetime.
- Refresh concurrency, uncertain provider outcomes and current-credential
  publication under existing replacement/selection authority.
- Live selection/credential replacement and authority-withdrawal effects,
  including work while disconnected.
- Server refresh implementation must respect the repository's external-provider
  integration policy; the existing device-acquisition exception alone does not
  approve an arbitrary new refresh transport.
- No real credentials, native authentication, refresh or model calls have been
  exercised by accepting this direction.

## D25. Defer Saved Authentication Selection Changes to the Next Harness Start

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: option 1, saved selection with deferred application,
  rather than selection-triggered retirement of the running harness.
- Applies to: REQ-1, REQ-2, REQ-11, REQ-12 and REQ-17,
  Requirements revision 16.

### Decision

- Permit the currently authorized Agent owner to save a different eligible
  authentication selection while its harness is live.
- Keep the live harness on its previously applied authentication. Saving the
  selection alone does not terminate, restart or relogin that harness.
- Apply the saved selection when a genuinely new harness process starts. The
  next message, model turn and normal task completion are not that boundary.
- Expose the saved-versus-running authentication distinction without exposing
  credential bytes. Save success is not native application success.
- Do not add a dedicated selection-triggered shutdown/restart mechanism. The
  requester chose deferred application to avoid that additional lifecycle work.

### Boundaries and Remaining Work

The server-saved selection remains authoritative for future starts; the
execution's applied authentication is the source of its running configuration.
This split is not permission to replace saved selection from native local state.
D24's centrally owned refresh direction remains applicable.

- Current owner/Team, credential-scope and execution authority remain required.
  Deferred selection does not mean indefinite use after authority withdrawal.
- Define how the old applied credential remains eligible for renewal, and how
  catalogue deletion interacts with a running reference. Existing catalogue
  reference checks alone do not establish this new runtime relationship.
- Same-record API-key replacement, same-account reauthentication and automatic
  refresh are not decided merely by selecting this saved-reference policy.
- Existing D1/D14 retirement obligations remain; only selection-triggered
  retirement is excluded. No automatic uncertain-work replay is authorized.
- The Design must ground applied/saved identity checks and stale delivery fences
  without introducing an unapproved general versioning system.

## D26. Replace Credential Files Without Manipulating the Live Harness

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: automatic file replacement only, rather than deferring
  publication to a new start or forcing live reauthentication.
- Applies to: REQ-1, REQ-11, REQ-12, REQ-17 and REQ-18,
  Requirements revision 17.

### Decision

- For same-record API-key replacement or same-account ChatGPT reauthentication,
  automatically publish the new admitted credential material by replacing the
  native credential file for executions using that authentication record.
- Do not terminate/restart the harness, call re-login or force a native
  credential reload to make it adopt the file.
- File publication success proves only publication. Native credential-cache
  rereads and actual authentication success are separate facts. Do not report
  immediate native adoption merely because the file has changed.
- Keep D25's different-record saved-selection changes deferred to the next
  actual harness start. Update the file associated with the applied record, not
  an unrelated newly selected record.
- D24 still owns ChatGPT refresh centrally. Materialized native token data must
  exclude the refresh token, including after manual reauthentication.
- Deliver only under current owner/Team, record-scope and exact execution
  authority. A delayed old delivery must not overwrite a newer credential or
  another applied-record file.

### Feasibility and Remaining Scope

Inspected native code loads authentication files and caches auth state; external
file changes are not generically an immediate live-application guarantee.
The Design must validate the exact supported API-key/ChatGPT file format and
normal native reread behavior. File-only publication does not authorize an
adapter fork or a hidden fallback to forced reload if adoption is delayed.

Define safe file replacement, confidential transport, storage permissions,
credential-file lifetime/residue and reconnect publication without exposing
secrets. Storage-location/persistence choices are not selected just by naming
the credential-file mechanism.

The approved automatic-refresh direction under D24 remains, but its full native
adoption path is still conditional. Do not claim that file replacement alone
establishes seamless refresh during live work. Report any conflict between
required renewal behavior and actual stock native behavior.

Authority withdrawal/deletion is separate from credential replacement and
remains unresolved. This decision does not promise immediate offline revocation.
No real credential publication or native-authentication tests have been run.

## D27. Store Native Credential Files in Existing Computer Saved Storage

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, existing saved volume/PVC rather than an additional
  ephemeral credential mount.
- Applies to: REQ-1, REQ-11, REQ-17 and REQ-18,
  Requirements revision 18.

### Decision

- Place native credential files in an execution-appropriate native state
  directory on the Computer's existing saved volume/PVC.
- Retain read-only root filesystem, non-root execution, dropped capabilities
  and existing mount ownership/security constraints. Do not add a separate
  credential tmpfs/ephemeral mount for the initial integration.
- Restrict file access and bind publication to the intended Computer, native
  context and applied authentication. File permissions are not same-identity
  code isolation.
- API keys and required short-lived ChatGPT tokens may exist as plaintext in
  these files. They can remain on saved storage after power-off, crash or
  execution-environment replacement. Do not claim native file encryption or
  guaranteed erasure from server-side catalogue encryption.
- ChatGPT refresh tokens remain server-only under D24. D26's automatic file
  replacement without harness manipulation remains the update policy.

### Rationale and Evidence

Current Docker and Kubernetes workload profiles supply one writable saved
volume/PVC and a read-only root filesystem. Their exact profile validation does
not already allow an extra temporary mount. Reusing saved storage avoids that
additional profile and native-context separation work.

ACP authentication does not determine persistence: inspected API-key login
passes native credential-store configuration to the storage backend. Native
file storage writes `CODEX_HOME/auth.json`; keyring/auto/ephemeral are other
native modes, not the selected replacement for this file-based design.

### Remaining Implementation and Validation Boundaries

- Specify the concrete native directory/file ownership and safe replacement
  within saved storage, without modifying unrelated user login files.
- Define startup handling of leftover files under current selected/applied
  authority. Residue is not permission to adopt ambient credentials.
- Validate actual native token reread/refresh behavior, secret-safe diagnostics,
  duplicate/late publication and retaining native model context independently
  from credential updates.
- Confidential server-to-Connector delivery remains separate from disk storage.
  Selecting saved storage does not authorize public plaintext secret delivery.
- No new credential mount, live authentication, file-publication or residue
  tests have been executed by accepting this decision.

## D28. Retain the Existing Trusted-Network Credential-Delivery Boundary

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, reuse the current internal HTTP component route,
  rather than adding TLS or a separate protected-delivery mechanism.
- Applies to: REQ-1, REQ-17 and REQ-18, Requirements revision 19.

### Decision

- Deliver admitted API keys and required short-lived ChatGPT token material
  through the existing explicitly trusted internal component-network boundary.
- Reuse exact execution/Connector/session admission and current owner/Team and
  applied-record eligibility checks. Possession of a target ID is not authority.
- Do not add a public credential-distribution endpoint, relay, alternate model
  proxy or new encryption/key-exchange protocol for the initial integration.
- This HTTP path has no transport-confidentiality guarantee against network
  observers. Bearer authentication is not encryption. Operators must treat that
  network and its observers as part of the selected trust boundary.
- D24 still keeps ChatGPT refresh tokens on the server. Browser DTOs, transcript
  data, event copies and ordinary logs must not receive credential bytes.

### Relationship and Validation

D28 resolves the server-to-Connector delivery trust choice left open by D24/D27.
It does not authorize sending the same payload over an arbitrary public route or
claim protection from hostile same-network/same-identity code.

Validate current-source routing, target mismatch/revoked execution rejection,
operational-key rotation/reconnect, stale credential publication and secret-safe
errors with synthetic markers. The network trust decision is accepted policy,
not evidence that the not-yet-implemented delivery RPC has passed those tests.

## D29. Retain the Entire Current Captured Snapshot

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, complete current-snapshot retention rather than
  pruning captured events after server acknowledgement.
- Applies to: REQ-11, REQ-12 and REQ-16, Requirements revision 20.

### Decision

- Retain all captured events in the current snapshot's local JSONL copy,
  including events the server has already stored.
- Do not shorten the current source when advancing the server's retained
  snapshot/event cursor or receiving storage acknowledgement.
- Keep the current source until actual source-snapshot replacement or authorized
  deletion of its storage. This does not add a new user-facing reload action.
- Do not add automatic acknowledged-prefix pruning, rotation, compression or
  age-based expiry of the current captured snapshot in the initial integration.
- Preserve D18 catch-up and D19 rebuilding from the available current source,
  without a new prefix-expired recovery contract.

### Limits and Verification

The copy consumes the existing saved storage and may exhaust available capacity.
This is not an unlimited-capacity promise, an fsync/crash-loss guarantee or
native-side missed-event repair. D23's small-cost recording-failure qualification
remains in force; do not add a comprehensive storage manager to enforce this
retention choice.

This decision covers captured session events, not managed artifact cache
collection or a separate historical snapshot archive. Validate reconnect from
earlier retained cursors and server ACK advancement without source deletion.
Actual source replacement still follows the accepted replacement boundaries.

## D30. Clear Prior Session Data Before Filling Its Replacement

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, direct clear-and-refill rather than retaining a
  separate old dataset until staged atomic publication.
- Applies to: REQ-2, REQ-5, REQ-10 through REQ-13 and REQ-16,
  Requirements revision 21.

### Decision

- Apply D19's complete replacement boundary by clearing the previous affected
  session conversation data before filling the new current source.
- Use bounded ingestion of the available captured copy and retain its committed
  progress. Do not add a second staging/archive/swap dataset for the initial
  integration.
- An incomplete replacement is not a completed empty conversation. Show ordinary
  conversation loading/error affordances; internal source-reconstruction terms
  are not normal product labels.
- Do not accept new effect-bearing input or form/permission responses against an
  incomplete source. Keep exact known current-work Stop/shutdown control separate
  rather than making data loading permission to continue unwanted work.
- On transfer failure retain truthful incomplete/failure progress. Continuing
  captured-event transfer is not replaying user commands or external effects.

### Relationship and Validation

D30 resolves D19's publication choice. D29 keeps the current captured source
available within existing storage limits; no old independent product-fact archive
is preserved to bypass D19. A first no-cursor binding is not loss of a prior
snapshot and must preserve the accepted input that initiated setup.

Validate data removal scope, initial versus replacement binding, intermediate
read/input admission, ordinary loading/error UI, captured-progress continuation
and late old-source rejection. The selected strategy is not evidence of successful
rebuild or permission to display a backend enum as user-facing copy.

## D31. Default Unconfigured Agents to Managed Preparation

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, managed preparation without a prior explicit supply
  choice, rather than blocking first use until a choice is saved.
- Applies to: REQ-1, REQ-14 and REQ-15, Requirements revision 22.

### Decision

- Resolve an absent saved Agent supply selection to managed preparation under
  D2–D7. Owners wanting existing Codex use select that mode and its explicit
  absolute executable path under D8/D9/D15.
- Resolve the default from server-authoritative selection; do not infer it from
  ambient binaries, local authentication or harness-authored settings.
- Prepare on the first actual input that requires a genuinely new harness under
  D12. Reading a screen neither prepares/starts the harness nor powers on a
  Computer.
- An explicit existing-installation selection remains authoritative on failure;
  do not reinterpret its missing/ineligible CLI as permission for managed supply.
- Preserve D11's next-start application and saved-versus-applied distinction.
  This choice selects neither cache cleanup nor an apply-now/restart control.

### Relationship and Validation

D31 closes the initial-default choice left open by the owner-controlled supply
decisions. It adds no new release source, authentication fallback or execution
authority. Validate absent versus explicit managed/existing selection, initial
input versus reads, Computer-off behavior and existing-CLI failure without a
silent supply switch. Actual installation/runtime acceptance remains separate.

## D32. Collect Only Inactive Superseded Managed Installation Cache

- State: Accepted by the requester on 2026-11-09 (KST).
- Selected alternative: A, bounded-scope automatic cleanup rather than retaining
  all old installation cache until separately authorized storage deletion.
- Applies to: REQ-1, REQ-11 and REQ-14, Requirements revision 23.

### Decision

- After preparation publishes a complete eligible replacement combination,
  automatically collect unused superseded Chapchoo-owned installation cache.
- Protect active and preparing artifacts and the latest complete eligible
  combination. Shared runtime/adapter/native files needed by a protected
  combination remain protected even if an older combination also references them.
- Keep protection valid across preparation, native-start binding and retirement;
  cleanup cannot remove files that concurrent preparation/start is using.
- Do not remove external CLI installations, captured JSONL/session sources,
  native context, credential files or user workspace through cache cleanup.
- A previously removed combination may require reacquisition if needed again.
  Do not add a numeric quota, age-retention policy or general storage manager.

### Relationship and Validation

D32 closes inactive-artifact collection without changing D2–D7 eligibility,
freshness-failure fallback or D3's live-artifact binding. D29 governs captured
session-source retention independently. Incomplete/failed replacement is not a
completed replacement that permits collecting the protected previous combination.

Validate complete versus partial replacement, active/preparing/latest protection,
shared components, preparation/start/cleanup races and the owned-cache deletion
boundary with deterministic fixtures. Cleanup does not prove sufficient remaining
disk capacity or authorize touching an otherwise protected data class.

## D33. Keep Authentication Deletion Based on Saved References

- State: Confirmed by the requester on 2026-11-09 (KST).
- Selected outcome: allow otherwise-eligible deletion even while a prior applied
  authentication remains in an existing harness; no runtime-only deletion block.
- Applies to: REQ-1, REQ-17 and REQ-18, Requirements revision 24.

### Decision

- Preserve existing catalogue deletion admission: saved Agent references can
  return `InUse`; an applied-only runtime identity does not add another blocker.
- A D25 selection change does not itself revoke the old record. While it exists
  and remains authorized, D24/D26's same-record renewal and file-update rules
  continue to apply to the exact existing execution.
- After record deletion or loss of its current scope authority, deny new renewal
  admission and credential delivery. Do not retain a deleted secret for future
  refresh or reinterpret the new saved selection as applied to the running owner.
- Revalidate current record and execution authority before result publication
  and delivery. A late in-flight result cannot recreate a deleted record or
  establish new distribution authority.
- Record deletion does not immediately revoke already-distributed provider
  credentials or prove native-process termination. Existing Team/Agent/Computer
  authority-withdrawal paths remain in force.

### Relationship and Validation

This resolves the applied-only deletion clarification under D25, not a new
authentication mode or runtime-reference catalogue. D24 refresh ownership,
D26 file-only publication and D27 persistent-residue limits remain unchanged.
Validate saved-reference protection, permitted applied-only deletion, denied
post-deletion renewal/delivery, late-result rejection and no silent credential
switch. Already-forwarded material and provider effects remain subject to the
accepted delivery/uncertainty boundary, not retroactive revocation.

## Selected Evidence

- [Connector process](../spec/domain/connector-process.md): implemented connection
  ownership and its current observation-only limit.
- [Control call lifetime](../spec/domain/computer-control-rpc.md#stream-and-call-lifetime):
  transient correlation and uncertain effects after link loss.
- [ACP v2 session lifetime and synchronization](https://agentclientprotocol.com/protocol/v2/session-setup):
  session context and connection-active lifetime are distinct.
- [Codex ACP 2.5.0 producer source](https://github.com/agentclientprotocol/codex-acp/tree/34e0897/src):
  native session resume and state projection are interoperability evidence, not
  proof that Chapchoo already provides the selected lifecycle.
- [Official adapter distribution metadata](https://registry.npmjs.org/@agentclientprotocol%2fcodex-acp/2.5.0)
  and [Linux native Codex metadata](https://registry.npmjs.org/@openai%2fcodex/0.165.0-linux-x64):
  inspected artifact identity, integrity and attestation-field examples, not
  evidence of performed artifact or signature verification.
