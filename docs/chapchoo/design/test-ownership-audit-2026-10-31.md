---
title: "Repository-wide test ownership audit"
created: 2026-10-31
tags: [testing, quality, rust, frontend]
document_role: supporting
document_type: supporting-audit-report
---

# Test ownership audit — 2026-10-31

## Scope and method

Requester: tests verify the subject's own behavior; consumers do not re-test dependencies. This explicitly includes Rust and shared UI. A common component asserting `lg` → `16px` is still a consumer of token resolution and is not appropriate.

Baseline: `f2a3f365ecf7db37d4e03081703859fc8a3191c5`. The initial inventory contained 85 test-bearing files; a second content scan found the nonstandard `check-mascot-motion.mjs`, making **86 reviewed files**. Discovery covered tracked unit/component/E2E/support tests, inline Rust tests/doctests, custom Node check runners and executable Python acceptance scripts. Python/Swift reserved product directories had no additional tests. Fixture/helper files without their own assertions are not counted as test subjects.

Every inventoried assertion family was classified by implementation owner. Questionable cases were checked against their implementations. This is an ownership audit, not proof of complete behavioral coverage or a security audit. Retained does not mean perfect test design in every unrelated respect.

## Corrections

- Shared custom CSS, frame/scroll, growing header and custom menu interactions run once under the UI owner's independent fixture Storybook, rather than app suites. Pure authentication-client tests move to web-core.
- Generated Prost/ts-proto codec self-tests, library randomness checks, direct test-only Lettre TLS configuration, browser cookie arrival-order semantics and a fake backend reordering model are removed.
- Dependency validation matrices and declaration repeats are trimmed. Asset staging verifies copying selected source bytes, not PNG/WOFF internals.
- Actual adapter admission/mapping, schema/transaction invariants, resource ownership/cancellation, production TLS-policy selection, server/client composition and three core product journeys remain.
- Table PR #32 removes its newly added pixel/token tests; the setting is not replaced by an assertion that repeats the declaration.

## Reviewed inventory

### Rust (29 files)

- `rust/apps/chapchoo/src/config.rs` — Retained: owned configuration admission, process-group supervision, readiness, startup failure and shutdown composition.
- `rust/apps/chapchoo/src/web.rs` — Retained: owned configuration admission, process-group supervision, readiness, startup failure and shutdown composition.
- `rust/apps/chapchoo/tests/cli.rs` — Retained: owned configuration admission, process-group supervision, readiness, startup failure and shutdown composition.
- `rust/apps/chapchoo/tests/web_cli.rs` — Retained: owned configuration admission, process-group supervision, readiness, startup failure and shutdown composition.
- `rust/crates/crypto/src/code.rs` — Retained: our code representation, rejection-sampling/binding or sanitized error contract, not dependency cryptographic primitives.
- `rust/crates/crypto/src/error.rs` — Retained: our code representation, rejection-sampling/binding or sanitized error contract, not dependency cryptographic primitives.
- `rust/crates/crypto/src/password.rs` — Trimmed dependency salt-freshness assertion; retained our full-input admission, work-factor policy and redaction.
- `rust/crates/crypto/src/root.rs` — Trimmed dependency nonce uniqueness and AEAD tamper matrix; retained our session/digest/key binding and envelope admission.
- `rust/crates/crypto/src/token.rs` — Removed dependency random-output inequality; retained our canonical bearer representation, digest matching and redaction.
- `rust/crates/domain/src/email.rs` — Reduced library grammar matrix to our failure mapping and configured no-display-name policy; retained normalization/redaction.
- `rust/crates/domain/src/lib.rs` — Retained: domain-owned password/role/normalization or module authority constraints.
- `rust/crates/domain/src/password.rs` — Retained: domain-owned password/role/normalization or module authority constraints.
- `rust/crates/domain/src/role.rs` — Retained: domain-owned password/role/normalization or module authority constraints.
- `rust/crates/mail/src/lib.rs` — Removed the positive TLS test that built a test-only Lettre transport directly. Production-constructor envelope/body mapping, no replay, TLS-mode admission and redacted errors remain.
- `rust/crates/persistence/src/sessions.rs` — Retained: owned SQL schema constraints, atomic changes, authority, session rotation and cancellation/lease retirement across the two supported database adapters.
- `rust/crates/persistence/src/store.rs` — Retained: owned SQL schema constraints, atomic changes, authority, session rotation and cancellation/lease retirement across the two supported database adapters.
- `rust/crates/persistence/tests/authentication.rs` — Retained: owned SQL schema constraints, atomic changes, authority, session rotation and cancellation/lease retirement across the two supported database adapters.
- `rust/crates/persistence/tests/preparation.rs` — Retained: owned SQL schema constraints, atomic changes, authority, session rotation and cancellation/lease retirement across the two supported database adapters.
- `rust/crates/persistence/tests/teams.rs` — Retained: owned SQL schema constraints, atomic changes, authority, session rotation and cancellation/lease retirement across the two supported database adapters.
- `rust/crates/protocol/src/lib.rs` — Removed two generated Prost roundtrip tests; retained handwritten secret-safe Debug and dependency-boundary doctest.
- `rust/crates/rpc/src/ingress.rs` — Retained: owned admission, domain/wire mapping, sanitized status metadata and listener-surface integration; not generated codec algorithms.
- `rust/crates/rpc/src/mapping.rs` — Retained: owned admission, domain/wire mapping, sanitized status metadata and listener-surface integration; not generated codec algorithms.
- `rust/crates/rpc/src/status.rs` — Retained: owned admission, domain/wire mapping, sanitized status metadata and listener-surface integration; not generated codec algorithms.
- `rust/crates/rpc/tests/listeners.rs` — Retained: owned admission, domain/wire mapping, sanitized status metadata and listener-surface integration; not generated codec algorithms.
- `rust/crates/service/src/password.rs` — Retained: owned orchestration, failure mapping, mail-after-commit, acceptance time, configuration and worker-capacity lifecycle.
- `rust/crates/service/src/time.rs` — Retained: owned orchestration, failure mapping, mail-after-commit, acceptance time, configuration and worker-capacity lifecycle.
- `rust/crates/service/tests/acceptance_time.rs` — Retained: owned orchestration, failure mapping, mail-after-commit, acceptance time, configuration and worker-capacity lifecycle.
- `rust/crates/service/tests/accounts.rs` — Trimmed delegated team membership, proof consumption and session revocation assertions already owned by persistence. Retained service-owned mail capability gating, issued-link path/lifetime construction, password-work composition/error mapping, mail-after-commit and code-result mapping.
- `rust/crates/service/tests/configuration.rs` — Retained: owned orchestration, failure mapping, mail-after-commit, acceptance time, configuration and worker-capacity lifecycle.

### Python tooling and acceptance (15 files)

- `.claude/skills/ci-optimization/scripts/analyze_e2e_ci_test.py` — Retained: tool-owned input/schema validation, scope matching, checkpoint transitions or analysis calculations. Roundtrips exercise our decoder/filtering, not JSON itself.
- `.claude/skills/convention-audit/scripts/audit_state_test.py` — Retained: tool-owned input/schema validation, scope matching, checkpoint transitions or analysis calculations. Roundtrips exercise our decoder/filtering, not JSON itself.
- `.claude/skills/living-spec-audit/scripts/audit_state_test.py` — Retained: tool-owned input/schema validation, scope matching, checkpoint transitions or analysis calculations. Roundtrips exercise our decoder/filtering, not JSON itself.
- `.claude/skills/skill-creator/scripts/quick_validate_test.py` — Retained: tool-owned input/schema validation, scope matching, checkpoint transitions or analysis calculations. Roundtrips exercise our decoder/filtering, not JSON itself.
- `.codex/hooks/claude_rules_hook_test.py` — Retained: tool-owned input/schema validation, scope matching, checkpoint transitions or analysis calculations. Roundtrips exercise our decoder/filtering, not JSON itself.
- `scripts/test-container-images.py` — Retained: executable cross-runtime/distribution integration of our selected surfaces, runtime mounts, isolation, packaged assets and owned cleanup.
- `scripts/test-installed-linux.py` — Retained: executable cross-runtime/distribution integration of our selected surfaces, runtime mounts, isolation, packaged assets and owned cleanup.
- `scripts/test-rpc-clients.py` — Retained: executable cross-runtime/distribution integration of our selected surfaces, runtime mounts, isolation, packaged assets and owned cleanup.
- `scripts/tests/test_build_artifacts.py` — Removed duplicate package-validator format/security matrix. Retained adapter fail-before-output wiring, selected outputs, overwrite admission and builder invocation.
- `scripts/tests/test_chart_contracts.py` — Retained: our catalog, packaging, chart, publication or CI-wiring contracts; no Helm/Python/Docker algorithm self-tests found.
- `scripts/tests/test_ci_structure.py` — Retained: our catalog, packaging, chart, publication or CI-wiring contracts; no Helm/Python/Docker algorithm self-tests found.
- `scripts/tests/test_docs_catalog.py` — Retained: our catalog, packaging, chart, publication or CI-wiring contracts; no Helm/Python/Docker algorithm self-tests found.
- `scripts/tests/test_package_linux.py` — Retained: our catalog, packaging, chart, publication or CI-wiring contracts; no Helm/Python/Docker algorithm self-tests found.
- `scripts/tests/test_setup_proof.py` — Retained: our catalog, packaging, chart, publication or CI-wiring contracts; no Helm/Python/Docker algorithm self-tests found.
- `scripts/tests/test_snapshot.py` — Removed assertion of another script's fixed WEB_MOUNTS tuple; retained publication identity, completeness and workflow ordering.

### TypeScript and support (42 files)

- `testenv/chapchoo/src/config/config.test.ts` — Retained: support-owned environment admission, private prerequisites, diagnostic redaction and built-export boundary.
- `testenv/chapchoo/src/safety.test.ts` — Retained: support-owned environment admission, private prerequisites, diagnostic redaction and built-export boundary.
- `testenv/chapchoo/test/exports.test.mjs` — Retained: support-owned environment admission, private prerequisites, diagnostic redaction and built-export boundary.
- `typescript/apps/admin-web/src/config/public-path.test.mts` — Retained: app-owned runtime configuration and Next deployment-mount composition.
- `typescript/apps/admin-web/src/features/admin-auth/components/AdminAuth.component.spec.ts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/apps/admin-web/src/features/admin-auth/methodState.test.mts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/apps/admin-web/src/features/admin-grants/components/AdminGrants.component.spec.ts` — Removed duplicate common frame-size/scroll tests; common UI owns those contracts.
- `typescript/apps/admin-web/src/features/admin-grants/revokeSelection.test.mts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/apps/admin-web/src/shared/appearance/Appearance.component.spec.ts` — Removed repeated common header geometry/menu behavior; shared UI owns it.
- `typescript/apps/admin-web/src/shared/auth/Authentication.component.spec.ts` — Retained: app-owned RPC/provider composition, initial identity handoff and mounted login/logout wiring; pure web-core behavior moved separately.
- `typescript/apps/admin-web/src/shared/lib/localization.test.mts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/apps/admin-web/src/shared/server/inventory.test.mts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/apps/e2e/src/accounts.spec.ts` — Retained: three assembled core product journeys and server/client handoff. Product integration is the subject, not isolated dependency layout.
- `typescript/apps/e2e/src/config/output.test.ts` — Retained: runner-owned private output path and filesystem permissions.
- `typescript/apps/e2e/src/cookie-order.spec.ts` — Removed browser arrival-order/HttpOnly experiments; moved actual application logout response integration to web-core/test/logout.browser.spec.mjs.
- `typescript/apps/web/src/config/public-path.test.mts` — Retained: app-owned runtime configuration and Next deployment-mount composition.
- `typescript/apps/web/src/features/accounts/accounts.test.mts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/apps/web/src/features/accounts/components/Accounts.component.spec.ts` — Moved custom typography/glyph-fit checks to UI; removed repeated common colors/frame tokens and font-distribution checks. Retained app-owned recovery, copy, navigation, character handoff and workspace composition.
- `typescript/apps/web/src/features/accounts/components/SharedLayout.component.spec.ts` — Moved shared size/scroll checks to UI; removed compact token-resolution assertion. Retained only app context-row/section alignment and reflow; removed shared background/padding assertions.
- `typescript/apps/web/src/features/accounts/proofPreview.test.mts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/apps/web/src/shared/appearance/Appearance.component.spec.ts` — Moved custom menu callback/draft/focus behavior to UI; removed Mantine arrow-key navigation and repeated per-page geometry matrix.
- `typescript/apps/web/src/shared/appearance/HeaderEnlargement.component.spec.ts` — Moved growing header geometry to UI with only owner fixtures; removed duplicate consumer fixture.
- `typescript/apps/web/src/shared/appearance/HeaderRecovery.component.spec.ts` — Kept app-owned mount continuity through recovery; removed a repeated common menu-selection/focus interaction.
- `typescript/apps/web/src/shared/auth/Authentication.component.spec.ts` — Retained: app-owned RPC/provider composition, initial identity handoff and mounted login/logout wiring; pure web-core behavior moved separately.
- `typescript/apps/web/src/shared/auth/client.test.mts` — Moved to web-core/test/auth-client.test.mjs, next to the sole implementation. No app-specific behavior was present.
- `typescript/apps/web/src/shared/lib/locale.test.mts` — Retained: app-owned selection, error/copy/locale mapping, input admission, recovery or server-selected character handoff; no independent dependency algorithm assertion found.
- `typescript/packages/design-reference/check-design.mjs` — Moved canonical authored-token/document and contrast checks to UI/check-design.mjs, the adapter owner; no Mantine token resolution is asserted.
- `typescript/packages/design-reference/check-mascot-motion.mjs` — Retained: reference-owned artwork mapping, authored grounded transform and playback/cancel state machine. Framework timing/animation implementation is not asserted.
- `typescript/packages/rpc-client/test/contract.test.mjs` — Removed five generated ts-proto codec/path self-tests and their empty test command; generation check, compilation and live cross-runtime integration remain.
- `typescript/packages/rpc-client/test/live-rpc.mjs` — Retained: live Node/Rust contract interoperability, per-call caller authority, typed ingress and selected listener surfaces on both stores; distinct from codec self-tests.
- `typescript/packages/ui/src/FontFaces.test.tsx` — Retained: owned runtime-path rendering, controlled presentation or hook/view connection; not Mantine/React internals.
- `typescript/packages/ui/src/Mascot.test.tsx` — Removed a static catalog-declaration repeat; retained runtime mount/default/selected-character rendering.
- `typescript/packages/ui/src/createReactContainer.test.tsx` — Retained: owned runtime-path rendering, controlled presentation or hook/view connection; not Mantine/React internals.
- `typescript/packages/ui/src/presentation.test.tsx` — Removed two field tests that repeated semantic defaults and plain prop forwarding through Mantine/React. Retained Feedback's owned loading/error/empty/loaded branch selection.
- `typescript/packages/ui/src/webIdentity.test.tsx` — Retained: owned runtime-path rendering, controlled presentation or hook/view connection; not Mantine/React internals.
- `typescript/packages/web-core/test/auth.test.mjs` — Removed fake backend overlap/reordering algorithm test. Retained actual RequestSession disposition, transport ingress, safe reporting and request isolation.
- `typescript/packages/web-core/test/color-mode.test.mjs` — Retained: web-core-owned config/admission, response disposition, SSR handoff, URL mapping, appearance bootstrap and explicit error boundaries.
- `typescript/packages/web-core/test/logout.test.mjs` — Retained: web-core-owned config/admission, response disposition, SSR handoff, URL mapping, appearance bootstrap and explicit error boundaries.
- `typescript/packages/web-core/test/page-read.test.mjs` — Retained: web-core-owned config/admission, response disposition, SSR handoff, URL mapping, appearance bootstrap and explicit error boundaries.
- `typescript/packages/web-core/test/page-state.test.mjs` — Retained: web-core-owned config/admission, response disposition, SSR handoff, URL mapping, appearance bootstrap and explicit error boundaries.
- `typescript/scripts/design-boundaries.test.mjs` — Retained: our import/manifest/style boundary analyzer against isolated fixtures.
- `typescript/scripts/stage-web-fonts.test.mjs` — Replaced source font/icon format/dimension assertions with staging-owned source/output tree and byte preservation checks; no third-party file-format behavior is re-tested.

## Verification

Local validation passed: Rust fmt/clippy/workspace tests (64 tests including doctests) and dependency boundaries; generated-client verification and live Node/Rust RPC on SQLite and PostgreSQL (71 checks per engine); frontend format/lint/typecheck/build; four fixture Storybooks; UI component checks (8), General (11), Admin (4), and web-core browser boundary (1). Python CI/build/publication/catalog/package/setup tests (52), chart checks (13), hook checks (8), and skill helper suites also passed. Unit/support commands cover each owning package plus asset staging.

Final-SHA CI and review results are reported on the PR. Deleted assertion counts are not treated as verification, and no merge/deployment is performed by this cleanup.
