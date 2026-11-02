---
title: "Agent Computer Phase 4 — Web, Deployment and Integrated Verification"
created: 2026-11-02
tags: [computer, implementation, frontend, deployment, testing]
---

# Phase Execution Plan

- Phase: 4 of exactly 4 substantive stacked PRs.
- Branch/base: `feat/computer-web-integration` → `feat/computer-control-runtime` (PR #42, `5d591684`).
- Inputs: approved Requirements r4, accepted ADR D1–D26, approved Design r2 M1–M22; submitted PRs #40/#41/#42.
- PR boundary: owner Agent list/create/detail/lifecycle UI; read-only Admin Computer Provider inventory; bounded visible-view observation; chart/package/testenv group/endpoint integration; minimum product E2E and current Specs.
- Deliverables: authenticated SSR primary content, server-derived state/actions with separate management/Connector/power/read-warning information, accessible consequential confirmations, no browser lifecycle scheduler, explicit group targets and namespace-scoped runtime deployment defaults.
- Non-goals: harness/conversations/filesystem UI, Team Agents, concrete Mac runtime, provider/adoption/cleanup mutation UI, merge/deployment, live cluster provisioning or production credentials.
- Interfaces: source-generated Computer RPC and existing request-local session/tRPC boundary; opaque Request ID preconditions; visible unary polling (5 seconds for pending work, 15 seconds stable, no background interval), no retries or queued writes; namespace-only supported Kubernetes effects.
- Approved Design mechanisms: M15/M16/M18/M20/M21; integration coverage of M1–M14/M17/M19/M22 without new authority.
- Authority references: [computer-261102/REQ](../requirements/computer-261102-agent-computer-foundation.md), [computer-261102/ADR](../adr/computer-261102-agent-computer-foundation.md), [computer-261102/DESIGN](../design/computer-261102-agent-computer-foundation.md), current account web/RPC/delivery and Computer Specs.
- Design delta: None.
- Removal obligations: obsolete chart role/listener flags and broad health names, implied web target configuration, account-only package/testenv wiring. Preserve existing account/team/auth/display contracts and unrelated work.
- Absence verification: render chart combined/split/remote-web cases; search obsolete role flags/service names; verify generated-client boundaries and original account acceptance coverage.

| Workstream                      | Owner | Paths                                                      | Dependencies                      | Validation                                                                                                   |
| ------------------------------- | ----- | ---------------------------------------------------------- | --------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Web transport and DTO boundary  | /root | web-core, app server routers/loaders, generated client use | PR3 RPC and session authority     | DTO validation, typecheck and boundary tests                                                                 |
| Owner screens and observation   | /root | web features/routes/messages/navigation                    | Typed DTOs, server projection     | Static meaningful stories, component checks, SSR and minimum product E2E                                     |
| Read-only operator inventory    | /root | admin-web features/routes/messages/navigation              | Redacted inventory RPC            | Read-only contract, no private content, SSR/error/pagination checks                                          |
| Distribution                    | /root | infra/charts, images, testenv, package scripts and docs    | Runtime groups/component commands | Offline Helm policy/RBAC/target tests, package tests; live gates NOT EXECUTED without approved prerequisites |
| Integrated validation and Specs | /root | current Specs and phase plans                              | Stable phase4 integrated diff     | Rust/RPC regression as affected; TS quality/build/tests; minimum E2E; then complete stack CI/review          |

- Integration order: transport/DTOs → SSR loaders and browser observation → owner/operator pure screens and containers → chart/package/testenv → minimum E2E → Specs → PR4 → stack CI/independent/automatic reviews and batch corrections.
- Independent review: exact `/root/computer-reviewer`, read-only one-way complete stable phase review, requested only after all four PRs exist per requester direction. Root makes all corrections.
- Final validation: root-run generated RPC, affected Rust checks, TypeScript format/lint/typecheck/unit/build, stories/component checks, offline chart/package contracts and minimum core E2E. Distinguish local/synthetic evidence from live cluster CNI/storage/network/retirement gates.
- Scope-drift check: compare both missing approved behavior and unauthorized additions against M1–M22. Preserve existing account data and display/localization. Do not promote Requirements/Design to implemented while required evidence remains missing.
- Context checkpoint: PR3 submitted at `5d591684` with Rust107, Clippy/boundaries, generated Node, live RPC67 per DB and normal hooks passing. No remote CI/reviews checked yet. Owner and operator SSR/DTOs, confirmations bound to their opening request, identity-loss content withdrawal, bounded polling, explicit group/target distribution, Provider namespace RBAC and internal peer policy are implemented. The runtime sweep matches the approved 15-second default. All four source product E2E journeys pass; Rust107/Clippy, web-core60, web25/Admin8 component checks and offline chart15 pass before final focused/hook validation. Independent/automatic review and exact-head CI remain deferred until PR4 is submitted. No live cluster gate has run. Current Specs are integrated; Requirements/Design remain unpromoted while completion gates are pending.
