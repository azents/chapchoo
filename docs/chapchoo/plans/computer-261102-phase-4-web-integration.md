---
title: "Agent Computer Phase 4 — Web, Deployment and Integrated Verification"
created: 2026-11-02
tags: [computer, implementation, frontend, deployment, testing]
---

# Phase Execution Plan

- Phase: 4 of exactly 4 substantive stacked PRs.
- Branch/base: `feat/computer-web-integration` → `feat/computer-control-runtime` (PR #42, `9b62e101`). The original `5d591684` parent and `04823401` phase tip were preserved before exact-tip stacked rebases.
- Inputs: approved Requirements r4, accepted ADR D1–D26, approved Design r2 M1–M22; submitted PRs #40/#41/#42.
- PR boundary: owner Agent list/create/detail/lifecycle UI; read-only Admin Computer Provider inventory; bounded visible-view observation; chart/package/testenv group/endpoint integration; minimum product E2E and current Specs.
- Deliverables: authenticated SSR primary content, server-derived state/actions with separate management/Connector/power/read-warning information, accessible consequential confirmations, no browser lifecycle scheduler, explicit group targets and namespace-scoped runtime deployment defaults.
- Non-goals: harness/conversations/filesystem UI, Team Agents, concrete Mac runtime, provider/adoption/cleanup mutation UI, merge/deployment, live cluster provisioning or production credentials.
- Interfaces: source-generated Computer RPC and existing request-local session/tRPC boundary; opaque Request ID preconditions; visible unary polling (5 seconds for pending work, 15 seconds stable, no background interval), no retries or queued writes; namespace-only supported Kubernetes effects.
- Approved Design mechanisms: M15/M16/M18/M20/M21; integration coverage of M1–M14/M17/M19/M22 without new authority.
- Authority references: [computer-261102/REQ](../requirements/computer-261102-agent-computer-foundation.md), [computer-261102/ADR](../adr/computer-261102-agent-computer-foundation.md), [computer-261102/DESIGN](../design/computer-261102-agent-computer-foundation.md), current account web/RPC/delivery and Computer Specs.
- Design delta: None.
- Review correction scope: deployment-neutral public config with mandatory runtime targets; actual Computer authentication/role-loss cache withdrawal; exact unresolved creation recovery independent of fresh Provider eligibility; normalized resource URL cursors; typed read-only Provider create capability; and owner confirmation of PR3's safe same-Agent preparation retry. Common account fixture Role/port migration is now owned by PR3; this phase retains Computer-specific fixture support.
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
- Integrated correction checkpoint: all four PRs were submitted before stack CI/review, as requested. PR3 at `9b62e101` passed Rust110/Clippy/boundaries, live RPC67 per database, both native architectures/packages and all nine exact-head CI checks; addressed automated threads were answered/resolved. PR4 correction validation passed web-core62, General13/Admin19 unit tests, format/lint/types, both actual deployment-neutral Next builds with no backend/public deployment variables, both Storybooks, Web27/Admin9 component checks and all four assembled product E2E journeys. The same `/root/computer-reviewer` cleared the material corrected auth/recovery/owner-action boundaries and confirmed Design delta None. Per-SHA remote results belong to the published PR checks/report rather than this source checkpoint. Current Specs are integrated; live Kubernetes retirement, real saved-PVC contents, CNI and component TLS gates remain unexecuted without approved disposable prerequisites. Requirements/Design are not promoted as fully verified and plans are retained while that required evidence is missing.
