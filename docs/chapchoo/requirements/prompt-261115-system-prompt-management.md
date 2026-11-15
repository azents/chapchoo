---
title: "Agent Session Configuration Refresh and System Prompt Management Requirements"
created: 2026-11-15
implemented: 2026-11-16
document_role: primary
document_type: requirements
snapshot_id: prompt-261115
tags: [agent, conversation, frontend, security, mcp]
---

# Agent Session Configuration Refresh and System Prompt Management Requirements

- Snapshot: `prompt-261115`
- Document reference: `prompt-261115/REQ`

## Problem

An existing Agent conversation can retain stale session-supplied configuration when its system prompt or MCP-provided tool declarations change. A prompt-only application exception does not address the shared session lifecycle. The owner also cannot persistently manage the system prompt or inspect its exact product-supplied sources. Saved configuration, a connected process and configuration actually adopted by an execution are different facts.

## Primary Context

### Primary Actor

The individual Agent owner, with current membership in the Agent's Team and the existing private-Agent access rights.

### Primary Scenario

The owner continues an existing Agent conversation after session-supplied configuration changes, including a saved system prompt edit or changed MCP tool declarations. The product refreshes the session naturally, preserving conversation context and already running work rather than requiring the owner to manage internal restarts. The owner can inspect the original product-supplied prompt content and its sources for selected work, including after the saved setting changes again.

## Supporting Scenarios or Effects

- Clear the owner-authored prompt while retaining provider-owned product and function prompts.
- Preserve entered content after an unsuccessful save and accurately report known save outcomes.
- Inspect historical execution evidence without editing it or starting another execution.
- Handle preparation failure separately from a temporary failure to read existing evidence.
- Edit the system prompt through the confirmed Save-only interface.
- Adopt changed session-visible MCP tools through the same lifecycle rather than a prompt-specific workaround.

## Goals

- Persist owner-authored system prompts without repeated ordinary-message setup.
- Keep editing and saving simple while preserving existing conversation context and running work.
- Make actual product-supplied originals and sources inspectable for each execution.
- Preserve existing ownership, membership and credential boundaries.
- Establish a shared configuration-refresh lifecycle for prompt and MCP declaration changes.
- Prioritize a prepared, responsive next request over minimizing the number of safe reexecutions.

## Non-Goals

- Preset creation, template marketplaces, prompt sharing or additional administrator access.
- Prompt meaning interpretation, conflict adjudication or automatic rewriting as product policy.
- Separate Apply, Cancel or old-prompt permission choices in the prompt editor.
- A prompt version browser, history restoration or rollback feature.
- Complete harness-internal prompt visibility or proof of model compliance.
- Conversation reset, forced task interruption or native restart as user-visible prompt-editing steps.
- Implementing additional native harnesses as part of this feature.
- A new MCP marketplace, arbitrary server-management UI, runtime/plugin platform or unrelated configuration system.
- A configuration version browser, rollback interface, or automatically authorized revision/epoch tracking mechanism.

## Requirements

### REQ-1. Persist owner-authored system prompts

The owner can read, edit, save and clear the persistent system prompt for an existing Agent.

**Acceptance criteria**

- Reopening the editor shows the successfully saved owner-authored content.
- A valid, authorized save replaces the saved owner-authored content directly. The last successfully committed save is the setting, even when another editor saved after this editor was opened. Do not reject it solely as a stale edit or add a save-conflict resolution flow.
- Saving empty content clears only the owner-authored setting; input is not mandatory.
- Existing Agents without an owner-authored prompt remain usable with no invented user content.
- Saving the setting does not rename the Agent, change its ownership, reset its conversation or stop already running work.

### REQ-2. Keep prompt ownership and scope distinct

Owner-authored settings, provider-owned product and function prompts, and execution-only additions retain their separate authorship and scope.

**Acceptance criteria**

- The editor modifies only the owner's persistent Agent setting.
- Product and function prompts remain provider-managed and cannot be edited or disabled through this editor.
- Execution-only additions do not become persistent owner settings.
- Clearing owner-authored content does not remove required product or function prompts.
- Prompt interpretation and conflict handling remain Agent behavior, not new product policy controls.

### REQ-3. Use the confirmed simple editing flow

The Agent detail provides a system prompt entry consistent with surrounding divider rows. Desktop editing uses a centered modal; mobile editing uses the available full-screen space.

**Acceptance criteria**

- The entry uses the current Chapchoo shared interaction treatment, typography, theme and alignment rather than a separate standalone visual system.
- The desktop modal provides a generous multiline editor without occupying the whole desktop viewport; mobile keeps a spacious full-screen editor.
- Save is the only submission action. Close/back provides navigation without an additional Cancel or Apply flow.
- No old-prompt permission choice is presented before later requests.
- Long content is readable and editable within the editor, with keyboard access, visible focus and no page-level horizontal overflow.
- Existing surrounding Agent profile, execution connection and Computer controls retain their responsibilities.

### REQ-4. Reflect saved changes in existing conversations

The product applies saved changes to later work in existing conversations, while preserving context and already running work. Applying the change is a product responsibility, not an extra user configuration step.

**Acceptance criteria**

- A successful save requires no separate Apply action, conversation recreation or user-selected restart.
- Existing context and already running work are preserved rather than interrupted to apply an edit.
- A new request is not silently executed with superseded owner-authored content when the saved change has not been prepared for that work.
- There is no user-controlled old-prompt fallback or permission that persists across later requests.
- Save confirmation describes the established save fact and does not falsely establish native adoption or model compliance.
- The applicable preparation boundary and native feasibility are established during technical design without relaxing these outcomes implicitly.

### REQ-5. Preserve truthful save and failure behavior

Editing feedback preserves user work and distinguishes established success from failure or an unresolved outcome.

**Acceptance criteria**

- An unsuccessful save retains entered content and permits another attempt through the same Save action.
- Pending submission cannot create unsafe duplicate saves.
- A failed read does not present an empty setting that can overwrite unread saved content.
- Missing confirmation does not become a fabricated success, a fabricated failure or permission to replay an uncertain effect.
- An established save success is not reversed merely because a later application or supplementary read fails.
- Relevant failure is communicated concisely at the affected task without adding application-policy choices.

### REQ-6. Inspect exact supplied prompt originals and sources

The owner can select an execution and inspect the exact product-supplied prompt originals and their sources as read-only execution evidence.

**Acceptance criteria**

- Evidence identifies the selected Agent and execution and distinguishes owner-authored, product-common, function and execution-only sources when present.
- Historical evidence remains the content supplied at that time after the current setting changes or is cleared.
- Current settings are never used to reconstruct historical supplied content.
- A source that was genuinely absent is distinguishable from missing evidence or a temporary read failure.
- Reading evidence does not execute work, change saved settings or mutate historical content.
- The view covers what the product actually supplies, not all harness-internal instructions, the complete assembled model context or guaranteed model obedience.

### REQ-7. Prepare evidence before new execution

An execution with unknowable product-supplied prompt originals or sources is not a supported successful execution state.

**Acceptance criteria**

- If the required originals and sources cannot be prepared and preserved, new execution fails preparation before starting.
- A temporary inability to read already retained evidence is a retryable read failure, not proof that the original evidence never existed.
- Read or preparation failure does not reset context, substitute current settings for history or replay an already started request.
- Request and evidence attribution cannot silently move to another Agent, conversation or execution.

### REQ-8. Preserve private access and responsive product conventions

Editing and inspection retain the current private-Agent and Team-membership boundaries.

**Acceptance criteria**

- Team administration or knowledge of an Agent identifier alone does not grant another owner's prompt access.
- Authorization is enforced for both mutations and historical evidence reads; access loss does not preserve mutation authority.
- Authentication credentials and private operational capabilities are not presented as prompt sources.
- Desktop and mobile retain the same ownership and save semantics; source text is not automatically translated or rewritten.

### REQ-9. Refresh session-supplied configuration through one common lifecycle

System prompt changes and changes to session-visible MCP tool declarations use a common product-managed refresh lifecycle rather than unrelated prompt-only restart exceptions.

**Acceptance criteria**

- A saved system prompt change and a changed tool declaration/catalogue that requires native session reconfiguration are both covered by the lifecycle.
- When a relevant change is known, start preparing its refresh proactively at a safe boundary rather than postponing reexecution solely until another request arrives. Additional safe reexecutions are acceptable when they reduce user-visible preparation latency.
- If a new request arrives before safe refresh is ready, accept and retain that request and continue it once automatically after the required configuration is safely prepared, without asking the owner to resubmit.
- Refresh alone does not disable the conversation input or Send action, reject an otherwise admissible submission, or require the owner to wait before submitting. Submitted requests remain accepted and queued while onward native execution waits for safe preparation.
- An actual preparation failure or loss of the required target or authority ends that attempt with truthful failure feedback and preserved input. Automatic continuation of a waiting, not-yet-forwarded request is not permission to replay a request that may already have reached native execution.
- The owner continues the same product conversation and does not need to manage adapter processes, create another conversation or choose an old configuration.
- Already started model work and tool effects are preserved; refreshing configuration is not permission to cancel, replay or duplicate them.
- Refresh success reflects adoption of the relevant supplied configuration, not merely successful settings persistence, process launch, reconnection or saved declaration.
- Failure or an unresolved refresh outcome does not silently establish use of the new configuration, authorize a fresh unrelated context or discard prior context/evidence.
- A tool implementation or transport credential change that does not alter the session-visible tool contract is distinguished from an actual session-reconfiguration need; the feature does not require restarting on every server-side change.
- The point at which new work may proceed, and what happens when refresh cannot safely complete, is specified during design without inventing user-visible policy as an implementation detail.
- The distinction between same-context resume, execution-process restart and native-context replacement remains explicit. This requirement does not itself authorize replacing an existing native context.

### REQ-10. Provide a general-purpose default Agent project

Each Agent has a persistent default working project for ordinary files and work.
System prompt synchronization uses that project's standard harness instruction
path rather than a prompt-only temporary directory.

**Acceptance criteria**

- Ordinary project files survive preparation and session refresh.
- Saved instruction updates and clearing reach later work through the selected
  harness's standard project instruction behavior.
- The product retains distinct original sources for its generated instructions.
- The common project and instruction contracts can be implemented by another
  harness without exposing Codex-specific protocol or file names as product
  behavior.
- The feature adds no project registry, conflict-resolution workflow, injection
  timing controls, or additional harness implementation.

## Fixed Constraints

- The confirmed feature term is **system prompt**; user-facing copy must not introduce a replacement product term for it.
- Keep the current Chapchoo shared visual system and existing native execution/authentication boundaries unless an explicitly accepted technical decision changes them.
- Settings persistence, actual native delivery and model compliance remain distinct claims.
- Existing user data and immutable historical evidence must not be discarded to introduce this capability.
- A technical limitation that contradicts these outcomes must be reported before changing the product contract.
- Refresh is a shared session-lifecycle concern, not a system-prompt-specific permission to replace an empty native session.
- Use supported, unmodified Codex native software. Modifying, patching or forking Codex itself is outside this feature's design scope.

## Open Assumptions

- The supported native integration can provide a credible way to apply changed prompts to later work without losing the existing conversation. This is a feasibility dependency, not a verified capability or permission to reset it.
- The technical meaning and identity of a prompt-bearing execution must be made explicit, including the relationship to already running work and steering. It must not weaken per-execution evidence or the existing-conversation requirement.
- Exact native application timing and text capacity remain unresolved. Concurrent saves use direct overwrite; implementation details must not introduce stale-edit rejection, automatic merging or a new versioning mechanism.
- Which MCP changes need session reconfiguration, and whether same-context resume or physical restart can adopt them, must be verified against the supported integration rather than assumed from the term "restart."
- The safe boundary for outstanding model work, background commands and tool effects remains a material lifecycle decision. Neither indefinite waiting nor forced cancellation is implicitly authorized.
- Proactive refresh reduces expected preparation latency; it is not a guarantee that every arriving request has zero delay. Accepted waiting requests continue automatically after safe preparation, but the observation bound and precise coordination mechanism remain unresolved.

## Confirmation

REQ-1 through REQ-8 and their original prompt-management scope were confirmed by the requester on 2026-11-15 at 16:22 KST. At 16:54 KST the requester broadened the objective to a common natural session-reexecution design covering system prompts and MCP tools. At 21:03 KST the requester confirmed the complete updated Requirements document, including the revised primary scenario and REQ-9. At 21:32 KST the requester prioritized avoiding user preparation latency over reducing the number of safe reexecutions. At 21:56 KST the requester confirmed retaining a new request during pending refresh and continuing it once automatically after safe preparation, with preparation failure, preserved input and no uncertain replay as explained. At 22:46 KST the requester selected direct overwrite for concurrent saved edits, without stale-edit rejection or a conflict-resolution flow. These clarifications are reflected in REQ-1 and REQ-9. Prior prompt-only context-replacement proposals remain unaccepted. These confirmations do not authorize implementation, deployment or unresolved material technical mechanisms.

At 23:31 KST on 2026-11-15, the requester explicitly confirmed accepting and
queuing user submissions during refresh while deferring only onward native
execution. Blocking user input itself because refresh is pending is outside the
confirmed behavior. This clarifies REQ-9; it does not approve the unresolved
native transition, context-restoration mechanism or implementation.

At 00:08 KST on 2026-11-16, the requester excluded modifying Codex itself as a
solution for this feature. Existing product outcomes remain unchanged; native
engine source changes, patches and forks are outside the design scope.

At 01:39 KST on 2026-11-16, the requester delegated remaining technical decisions
and authorized implementation and verification, prioritizing robust future
harness extension and maintainability for an open-source project. At 01:42 KST
the requester confirmed the general-purpose default-project/instruction-sync
direction, emphasizing simplicity. REQ-10 records that scope. At 01:43 KST the
requester excluded new conflict and injection-timing product controls.
