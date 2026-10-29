---
title: "Rust Foundation Convention Parity Review"
created: 2026-10-29
tags: [architecture, rust, conventions, research]
document_role: supporting
document_type: supporting-review
snapshot_id: accounts-261029
---

# Rust Foundation Convention Parity Review

## Purpose and Status

Evaluate the complete existing convention baseline, not six isolated tool categories.
Long-term adoption and community sustainability are major decision criteria.
This is evidence and candidate mapping, not a selected Rust stack or implementation approval.

Reference: azents/azents commit `e99ac29b6bbdc05ee7a75a03b78fca2047073205`,
verified as the reference main HEAD during this review. All 99 Markdown files
under `.claude/conventions/` were inventoried and read: 97 rule bodies plus
AGENTS.md and its CLAUDE.md pointer. Source rule text is input, not proof every
current implementation obeys it.

Classification: **retain** (same surface/principle), **adapt** (Rust equivalent),
**verify** (material enforcement/compatibility gap), **conditional** (only if
that tool/surface exists), **defer** (outside this cycle), **process** (meta).
None means silently discarded. Existing TypeScript, infrastructure, and delivery
rules stay in scope alongside Rust-specific translations.

## Rule-by-Rule Coverage

| Source under .claude/conventions/ | Disposition | Meaning and candidate enforcement |
| --- | --- | --- |
| `AGENTS.md` | process | Convention index generation/instructions remain documentation tooling, not a Rust runtime dependency. |
| `CLAUDE.md` | process | Convention index generation/instructions remain documentation tooling, not a Rust runtime dependency. |
| `backend/schema-dependencies-in-living-specs.md` | adapt | Living Specs record actual schema keys/FKs/delete/cleanup ownership with both migrated catalogs checked; probes are not implementation evidence. |
| `github-actions/pin-actions-to-commit-hash.md` | retain | Pin CI actions to full commit SHA and use normal dependency updates; include Rust gates without weakening existing web checks. |
| `global/avoid-oppositional-negative-rewrites.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `global/comments-and-docstrings-in-english.md` | retain | English source/rustdoc and maintained artifacts; localized product samples remain intentional. |
| `global/decode-json-into-typed-payloads.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `global/dependency-updates-without-overrides.md` | retain | Cargo normal resolution/update/tree; no patch/override to hide a constraint without approval. |
| `global/deterministic-test-synchronization.md` | retain | Tokio barriers/channels/Notify and explicit clocks; no sleeps or yields as success evidence. |
| `global/diagrams-in-mermaid.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `global/load-tests-as-reports.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `global/patch-null-means-set-null.md` | retain | Presence-aware protobuf/JSON adapters; nullable patch uses explicit unset/null/value states. |
| `global/prefer-supported-external-service-sdks.md` | retain | Select maintained public Rust SDKs and verify operation coverage before custom transports. |
| `global/redis-optional-no-ha.md` | retain | Keep authority durable; no Redis dependency is introduced in this cycle. |
| `global/remove-obsolete-feature-tests.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `global/runner-reported-agent-workspace.md` | retain | Retain when Runtime work is in scope; no fixed Runtime filesystem roots invented now. |
| `global/stacked-pr-rebase-script.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `global/user-facing-text-in-english.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `global/worktree-rebase-origin-main.md` | retain | Retain the language-independent rule; enforcement remains review/tests or the existing delivery tooling, not a blanket rustc guarantee. |
| `infra/argocd-valuesobject-full-replacement.md` | conditional | Keep when the referenced deployment tool/resource is used; do not introduce AWS/Terraform/ArgoCD/Fluent Bit merely to exercise a convention. |
| `infra/fluentbit-explicit-kubernetes-filter.md` | conditional | Keep when the referenced deployment tool/resource is used; do not introduce AWS/Terraform/ArgoCD/Fluent Bit merely to exercise a convention. |
| `infra/iam-role-description-ascii-only.md` | conditional | Keep when the referenced deployment tool/resource is used; do not introduce AWS/Terraform/ArgoCD/Fluent Bit merely to exercise a convention. |
| `infra/kustomize-namespace-must-match.md` | conditional | Keep when the referenced deployment tool/resource is used; do not introduce AWS/Terraform/ArgoCD/Fluent Bit merely to exercise a convention. |
| `infra/module-separation-by-role.md` | conditional | Keep when the referenced deployment tool/resource is used; do not introduce AWS/Terraform/ArgoCD/Fluent Bit merely to exercise a convention. |
| `infra/resource-naming-prefix.md` | conditional | Keep when the referenced deployment tool/resource is used; do not introduce AWS/Terraform/ArgoCD/Fluent Bit merely to exercise a convention. |
| `infra/snake-case-vs-camel-case.md` | conditional | Keep when the referenced deployment tool/resource is used; do not introduce AWS/Terraform/ArgoCD/Fluent Bit merely to exercise a convention. |
| `opencode/no-agent-temperature.md` | conditional | Keep if OpenCode agent definitions are used; no Agent configuration feature added to Chapchoo. |
| `python/alembic-revision-only.md` | adapt | Use the selected migration generator and review output; preserve migration metadata and readiness identity. Alembic itself is Python-specific. |
| `python/apps-depend-on-libs-only.md` | adapt | Cargo app/binary targets remain graph leaves; reusable crates must not depend on deployable apps. Verify cargo metadata. |
| `python/avoid-underscore-dependency-fields.md` | adapt | Use Rust visibility and named fields; concrete collaborators by default, traits only for a real capability/substitution boundary. |
| `python/bind-extra-for-shared-log-context.md` | adapt | Structured tracing spans bind shared context; per-event fields remain explicit. Do not hold entered span guards across await. |
| `python/boolean-no-is-prefix.md` | adapt | Keep wire/DB field naming consistent; evaluate Rust predicate-method idioms separately instead of banning is_* methods mechanically. |
| `python/cancellederror-separately.md` | verify | Rust future cancellation is often drop, not a catchable exception. Explicitly own tasks, joins, rollback and async cleanup; test cancellation safety. |
| `python/catch-narrowest-exception.md` | adapt | Match typed error variants narrowly; propagate unrelated source errors with Result rather than converting all errors into absence. |
| `python/dedent-llm-prompts.md` | defer | Readable raw strings or checked-in templates preserve prompt text; no prompt feature implemented in this cycle. |
| `python/dependency-injection-no-singletons.md` | adapt | Constructor/function injection and an explicit composition root; Arc shares ownership but is not a service locator. No DI container chosen. |
| `python/dependency-injection-over-post-init.md` | adapt | Build dependencies at composition root, pass collaborators to new; avoid hidden infrastructure creation during domain construction. |
| `python/dependency-version-pin.md` | verify | Translate reproducible apps and reusable libs to Cargo.lock, --locked, workspace dependency policy and Cargo semver. Do not copy uv lowest-direct or >= rules literally. |
| `python/engine-changes-all-lowerers.md` | defer | Preserve provider-neutral semantics and enumerate adapters when Engine work is introduced; do not add lowerers to account scope. |
| `python/exhaustive-union-match.md` | adapt | Closed enums and exhaustive match, without catch-all arms hiding new domain variants; validate unknown protocol enum values at ingress. |
| `python/explicit-named-indexes.md` | adapt | Explicit unique/index names and backend catalog checks; respect PostgreSQL identifier byte limits in ORM and migrations. |
| `python/import-from-defining-module.md` | adapt | Keep source ownership clear, but evaluate idiomatic Rust pub use facades and privacy; do not copy Python __init__ prohibition to lib.rs blindly. |
| `python/layered-architecture-no-skip.md` | verify | Keep protocol -> service -> complete repository operation. DB handles and transaction lifetimes remain repository-owned; compare crate graph/privacy/trait ports for enforcement. |
| `python/linear-migration-chain.md` | adapt | One ordered migration history; do not rewrite applied migrations. Preserve history during unexecuted branch reconciliation using selected tool semantics. |
| `python/llm-tool-schema-compatibility.md` | defer | Keep external schema compatibility distinct from internal enum expressiveness; tool schemas are future scope. |
| `python/log-errors-once.md` | adapt | Log at the handling boundary once; source-chain context propagation is not logging. Test structured output paths. |
| `python/logger-exception-for-errors.md` | verify | Rust error Display/source does not guarantee an origin backtrace. Select error reporting/backtrace policy explicitly; tracing alone is not Python traceback equivalence. |
| `python/never-discard-exceptions.md` | adapt | No let _ = fallible business work, broad .ok(), unwrap_or_default or discarded JoinHandle results hiding failures. |
| `python/no-empty-value-defaults.md` | adapt | Use required fields and Option/explicit state rather than empty sentinels; do not blanket derive Default for domain inputs. |
| `python/no-explicit-5xx-http-exception.md` | adapt | Separate expected domain failures from infrastructure failures; protocol boundary owns sanitized gRPC INTERNAL/reporting. Do not invent HTTP gateway statuses. |
| `python/no-getattr-setattr-type-bypass.md` | adapt | Typed structs/enums and validated adapters; Any/downcast, unsafe and unchecked casts are not routine fixes for missing types. |
| `python/postgres-enum-types.md` | verify | PostgreSQL native ENUM remains a real baseline constraint; SQLite needs an equivalent constrained representation. Do not mistake probe TEXT CHECK for accepted PostgreSQL schema. |
| `python/rdb-index-constants.md` | adapt | Named reusable schema identifiers in model/migration definitions; do not replicate Python class-constant syntax unnecessarily. |
| `python/required-new-fields.md` | adapt | Required struct fields expose callsite changes at compile time; Option can still require an explicit choice. Avoid defaults solely for compatibility. |
| `python/sphinx-style-docstrings.md` | adapt | Use rustdoc summaries, Errors/Panics/Safety sections where relevant and checked examples, without duplicate type prose. |
| `python/structured-logging-extra.md` | adapt | tracing structured fields and static messages; redact secrets and do not Debug-print credential DTOs. |
| `python/structured-multi-field-returns.md` | adapt | Named result structs for multi-field domain outcomes rather than positional tuples. |
| `python/timezone-aware-datetimes.md` | adapt | Explicit UTC/offset-aware instant types, not naive local datetime; select time/chrono with DB mappings and fake clock tests. |
| `python/toolkit-prompt-placement.md` | defer | Preserve minimal pre-discovery authority and typed context later; no toolkit implementation added now. |
| `python/transaction-db-operations-only.md` | adapt | Repository-owned DB-only transactions; no network/FS/broker work or injected callbacks crossing the transaction lifetime. |
| `python/transparent-error-handling.md` | adapt | Result<T,E> expresses real failure in Rust; propagate Err to transport status, never Ok containing a hidden unexpected error. |
| `python/type-safety-order.md` | adapt | Fix type/ownership model first; typed wrappers before erasure, unsafe, unchecked conversions or broad lint allow. Document narrow unavoidable exceptions. |
| `python/typeddict-for-update-apis.md` | adapt | Explicit patch field enum or presence-aware wire mapping preserves omitted/null/value; Option<T> alone may collapse required states. |
| `swift/apps-depend-on-libs-only.md` | adapt | Cargo crate direction and narrow reuse apply; Swift targets remain deferred. |
| `swift/declarative-views.md` | defer | SwiftUI details deferred; keep existing prop-driven TypeScript UI boundary. |
| `swift/exhaustive-state-enums.md` | adapt | Rust enums and exhaustive match preserve closed-state invariant. |
| `swift/explicit-actor-isolation.md` | adapt | Use Rust ownership, Send/Sync and task boundaries; not a claim that Arc<T> makes T thread-safe. |
| `swift/generated-project-inputs.md` | defer | Swift project generation deferred; preserve generated-input authority in Cargo/protobuf clients. |
| `swift/improve-types-before-suppression.md` | adapt | Fix ownership/types before erasure, unsafe or lint suppressions. |
| `swift/injected-collaborators.md` | adapt | Constructor injection, concrete collaborators by default, real substitution traits only. |
| `swift/swift-api-design.md` | adapt | Apply Rust API guidelines and visibility; do not copy foreign-language naming rules. |
| `swift/transparent-errors-and-cancellation.md` | adapt | Result propagation, task ownership and explicit cancellation cleanup, without success-shaped fallbacks. |
| `swift/typed-domain-boundaries.md` | adapt | Validated typed ingress, newtypes/enums, explicit patch presence and unknown-field policy. |
| `swift/value-semantics-first.md` | adapt | Owned structs/enums by default; Arc/reference identity only when justified; no speculative inheritance analogues. |
| `testenv/azents-web-cookie-names.md` | adapt | Use actual Chapchoo cookie policy in fixtures, not old literal Azents names; reference body differs from current production cookie source. |
| `testenv/e2e-core-flows-only.md` | retain | Representative assembled E2E; combinations in narrow unit/integration tests, including both databases. |
| `testenv/no-direct-db-write-in-tests.md` | retain | Assembled testenv creates state via product paths; isolated DB constraint probes are not reported as product E2E. |
| `testenv/qa-report-includes-slack-link.md` | defer | Slack integration is outside this cycle; preserve as future domain-specific QA evidence, not Rust stack selection. |
| `testenv/slack-password-login-uses-hidden-url.md` | defer | Slack integration is outside this cycle; preserve as future domain-specific QA evidence, not Rust stack selection. |
| `typescript/adt-state-types.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/apps-depend-on-packages-only.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/avoid-type-assertions.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/browser-persistence-hooks.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/container-component-pattern.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/cookie-based-locale.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/explicit-return-types.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/mantine-tokens-no-hardcoded.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/natural-technical-term-localization.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/no-direct-process-env.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/no-edit-generated.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/no-index-files.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/no-reverse-layer-imports.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/page-tsx-thin-entrypoint.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/pure-ui-stories.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/server-component-by-default.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/storybook-provider-structure.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/trpc-cache-invalidation.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/use-generated-public-client.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |
| `typescript/user-relevant-status-copy.md` | retain | Keep on the existing TypeScript web surface. Rust backend does not replace this rule; generated-client authority maps from OpenAPI to the accepted protobuf contract where applicable. |

## Findings That Block a Superficial Tool Selection

- A constructor and composition root can preserve dependency injection without a
  runtime DI container. A trait per struct is not required by the source baseline.
- Folder names alone do not enforce layer direction. Cargo graph boundaries,
  visibility, service ports, DB dependency restrictions and compile-fail checks
  must be evaluated together. Repository transactions must not leak to services.
- Strong Rust types do not validate arbitrary strings, decoded protobuf enum
  numbers, request authorization, or protocol presence semantics automatically.
- Result/source-chain propagation is not automatic originating-stack capture.
  Cancellation by dropping a Future is not Python CancelledError, and dropping a
  JoinHandle does not automatically stop its task. Demonstrate cleanup and failure.
- Native PostgreSQL ENUM and SQLite portability must be reconciled explicitly.
  The earlier minimal relational probe used TEXT CHECK for a narrow owner-FK test;
  it did not select that representation for the actual PostgreSQL schema.
- Python import/default/version-pin/docstring spelling is not portable authority
  for prohibiting idiomatic Rust modules or requiring incorrect Cargo ranges.
- Source conventions can lag implementation: cookie fixture names must follow
  the actual production policy rather than obsolete literal examples.

## Candidate Quality Gates, Not Yet Selected Configuration

### Accepted foundation direction and implementation candidates

| Concern | Candidate direction | Required limit or verification |
| --- | --- | --- |
| Dependency injection | Explicit constructors/functions and one composition root; concrete collaborators until a real port is needed | No service locator, runtime reflection, trait-per-class or DI-container dependency by default |
| Layering | Thin Tonic adapter, typed application service/ports, repository implementation and separate executable assembly | Runtime calls preserve API/service/repository ordering; compile-time dependency inversion keeps persistence out of adapters |
| Domain contracts | Structs/enums/newtypes, Result/Option and validated TryFrom conversions | Generated Protobuf/Serde shapes are not authorization/domain validation; patch presence remains explicit |
| Async | Tokio, owned tasks/joins and cooperative shutdown; bounded blocking work | Test cancellation and connection retirement; started bcrypt work is not abortable just because its caller is async |
| gRPC | Tonic/Prost generated client/server boundaries | Existing accepted protocol choice retained; generation must not be hand-edited, both client surfaces enforce current authority |
| Error reporting | thiserror-style typed domain errors; source-chain/report context at infrastructure boundaries | anyhow-style erasure only where heterogeneous reporting is intended; originating backtraces need explicit capture and secret filtering |
| Observability | tracing structured fields/spans and one owning error boundary | No entered guards across await, duplicate reports, credential Debug dumps or independent hidden Sentry call sites |
| DB | Diesel selected under D28; SQLite locally, PostgreSQL for deployment | Repository owns complete DB-only operations and leases; official pool adapter retires broken transactions; both-engine verification retained |
| Configuration/CLI | clap parsing and centralized typed configuration at composition | No scattered environment reads; private values do not become public web configuration |
| Dates and clocks | Offset/UTC-aware time/chrono type chosen with DB mapping, injected clocks for time contracts | Do not infer naive local values; tests distinguish clock time from synchronization |
| Lint/type/format | rustc, rustfmt, targeted Clippy with workspace inheritance | Warning policy actually applied to members; narrow justified exceptions instead of unsafe/cast/default escapes |
| Tests | cargo test baseline, nextest when its execution features justify inclusion; focused integration tests and representative E2E | No passing skips; deterministic barriers; assembled fixtures use product paths; actual PG behavior verified |
| Dependencies/build | Cargo.lock for app workspace, pinned supported toolchain, locked CI, reviewed feature graph; cargo-deny candidate | No hidden overrides or copied uv version rules; optional heavy tools justified by ongoing maintenance value |
| Documentation/generation | rustdoc and maintained Living Specs; generated protocol/code sources remain authoritative | Verify schema relationships/delete ownership and examples; no generated-file hand edits |
| Existing web/delivery | Retain TypeScript UI state/container/localization/Storybook rules and existing infrastructure/CI conventions | Rust does not replace the frontend or authorize new infrastructure/features |

The common direction is accepted under D29 and Diesel under D28. Optional tools
and exact versions in this evaluation are not independent approved features. Detailed
dependency versions, crate count and equivalent helper layout are not separate
product decisions. Backtrace capture, selected DB migration tooling, native enum
mapping and runtime lifecycle still require bounded stack-level verification.

- rustc/cargo check for compiled contracts and graph/type/lifetime checks.
- rustfmt for canonical layout; Clippy for selected correctness, suspicious,
  performance and project-specific restrictions. Do not enable every pedantic or
  restriction lint blindly or claim Clippy proves the architecture.
- Typed domain errors plus an error-reporting boundary with structured tracing
  and secret-safe context; choose backtrace handling explicitly.
- Tokio task ownership, cancellation and bounded blocking CPU work, with barrier-
  based tests. gRPC adapters and generated DTOs remain outside domain invariants.
- Public API rustdoc/tests and deterministic unit/integration/E2E layers; DB
  behavior is tested through the chosen library on both engines.
- Reproducible lock/toolchain handling and dependency/security/license review;
  additional tools must justify upkeep, not merely appear on a checklist.

## First-Party Practice Review and Remaining Work

### Verified examples

- **Vector**, commit `892ae938fb7c783dd8d1eaa44920ed7540076ed6`:
  [`Cargo.toml`](https://github.com/vectordotdev/vector/blob/892ae938fb7c783dd8d1eaa44920ed7540076ed6/Cargo.toml)
  centralizes workspace lint policy including `await_holding_lock`,
  `let_underscore_must_use`, and `undocumented_unsafe_blocks`.
  [`clippy.toml`](https://github.com/vectordotdev/vector/blob/892ae938fb7c783dd8d1eaa44920ed7540076ed6/clippy.toml)
  encodes forbidden APIs and disallows entered tracing span guards across await.
- Vector's
  [`RunningTopology`](https://github.com/vectordotdev/vector/blob/892ae938fb7c783dd8d1eaa44920ed7540076ed6/src/topology/running.rs)
  owns task handles/shutdown coordination; `stop(self)` transfers ownership into
  a completion future and tracks graceful outcomes. This supports studying task
  lifecycle, not copying topology/reload features into Chapchoo.
- **Apollo Router**, commit `f3e7e6c6accec2d41b40902c1b45975608a2c68b`:
  [`RouterToSupergraphRequestService<S>`](https://github.com/apollographql/router/blob/f3e7e6c6accec2d41b40902c1b45975608a2c68b/apollo-router/src/services/router/service.rs)
  stores a collaborator constrained by Tower Service/Send bounds and forwards
  readiness with typed async results. This is a real generic composition example,
  not evidence that a runtime DI container is needed.
- Apollo's
  [CI configuration](https://github.com/apollographql/router/blob/f3e7e6c6accec2d41b40902c1b45975608a2c68b/.circleci/config.yml)
  runs locked fetching/tests, rustfmt checking, and cargo-deny advisory checks.
  Its shared dependency manifest is evidence for workspace-level consistency.

### Official behavior and adaptation hazards

- [Rust's stabilization announcement](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/)
  establishes native async traits from Rust 1.75, with Send/dynamic-dispatch
  design constraints. A stale Async Book paragraph saying stable Rust lacks
  async traits must not drive the architecture.
- [Tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
  states started blocking tasks cannot be aborted and recommends limiting
  parallel CPU work. bcrypt and shutdown must account for this explicitly.
- [Rust module privacy](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
  enforces access, not automatic business-layer dependency direction.

### Community and maintenance weighting

Popularity is a major positive selection signal, not a cosmetic tie-breaker.
Evaluate usage, independent adopters, maintainers/activity, release continuity,
ecosystem integration, troubleshooting material and replacement cost. Download
counts overlap through CI/transitive dependencies; they are not unique users.
The initial comparison found substantial adoption for SeaORM, SQLx and Diesel.
SQLx has the largest observed download volume and is also used by SeaORM.
Diesel remains a serious candidate including its async extension. Convenience
alone is not sufficient to finalize SeaORM.

### Observed adoption indicators

The crates.io API was checked during this review. Cumulative download figures
are rounded and include CI/repeated/transitive downloads. They establish scale
of ecosystem use, not unique production deployments or a future support guarantee.

| Candidate | Observed cumulative downloads | Role and assessment |
| --- | --- | --- |
| Tokio | 1.12 billion | Widely adopted async runtime; integrated with the other shortlisted networking/DB tools |
| Tonic | 448 million | Established gRPC implementation; candidate for the already accepted transport |
| Prost | 671 million | Protobuf code generation/types; input invariants still require explicit validation |
| Serde | 1.61 billion | Established typed serialization boundary, not automatic business validation |
| tracing | 984 million | Established structured observability ecosystem; source backtrace capture is a separate concern |
| thiserror | 1.73 billion | Established typed-error derivation; candidate for domain/infrastructure source chains |
| anyhow | 1.10 billion | Established application-boundary reporting; do not erase domain errors across every layer |
| clap | 1.28 billion | Established CLI parsing; configuration still needs centralized validation |
| cargo-nextest | 13.5 million | Candidate test execution tool; native cargo test remains a supported baseline |
| cargo-deny | 6.9 million | Candidate advisory/license/source policy tooling, demonstrated in Apollo CI |
| shaku | 262 thousand | Much smaller DI-framework population; constructor injection can avoid requiring a container at all |

These tools solve different problems, so their download totals are not a common
performance or quality ranking. In particular a small DI crate is not directly
comparable to transitive serialization dependencies. The relevant conclusion is
that the proposed core runtime/type/logging/CLI direction can rely on a substantial
established ecosystem without adding a runtime DI framework.

## Isolated Compiled Foundation Probe

### Environment and commands

Installed user-space Rust tooling for this research, without modifying product
dependencies. Tool versions: rustc/cargo 1.99.1, Clippy 0.1.99, rustfmt 1.10.0.
The independent probe uses five crates: contracts, application, persistence,
protocol, and a composition/test harness. It uses Tokio 1.54.1, Serde 1.0.229,
serde_json 1.0.154, and thiserror 2.0.21 in its own lockfile.

Executed:

```console
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

Six tests passed. Clippy and rustfmt checks passed after formatting the probe.
Tests use explicit oneshot/channel boundaries rather than sleeps.

### Positive runtime evidence

1. The composition root injects a concrete memory store through an async-capable
   port into a service and route. A typed fake substitutes without a DI container.
2. Duplicate-account failure remains a typed error, not a success-shaped response.
3. Serde rejects unknown fields, wrong types, and missing required fields at ingress.
4. An infrastructure error preserves its source through the typed wrapper; this
   is not evidence of originating stack capture.
5. Dropping a JoinHandle leaves the task running; explicit cooperative stop plus
   awaiting its handle confirms cleanup completion.
6. A blocking task signaled as started continues after abort and finishes only
   after its explicit release. This reproduces the shutdown hazard relevant to
   CPU-bound bcrypt.

### Negative enforcement evidence

Each deliberately invalid case was rejected with the intended diagnostic; source
was then restored and the successful test/lint/format commands rerun.

| Invalid case | Actual rejection |
| --- | --- |
| Protocol imports persistence without a dependency edge | rustc E0433 |
| Return UserId where TeamId is required | rustc E0308 |
| Omit a closed AccountError variant in match | rustc E0004 |
| Discard a fallible filesystem Result with let underscore | Clippy let_underscore_must_use |
| Hold a std MutexGuard across await | Clippy await_holding_lock |

These checks demonstrate feasible enforcement mechanisms, not immutable policy:
someone can deliberately edit Cargo dependencies or suppress lints. A real
workspace must inherit its lint settings in members and validate its allowed
dependency graph in CI. More crates are not inherently better architecture.

### Limits and next gate

No production Rust source, Cargo workspace, framework convention configuration,
DB-library implementation, or product gRPC service has been created. The probe
does not test PostgreSQL/SQLite through SeaORM/SQLx/Diesel, generated protobuf
clients, originating backtraces, TLS/CSRF, deployment, or the complete convention
matrix. Existing documentation tests are separate evidence.

The candidate direction is explicit constructor composition, typed capability
ports only where justified, compiler-visible crate/privacy boundaries, Tokio,
Tonic/Prost, typed domain errors plus structured tracing, and rustfmt/targeted
Clippy/locked CI. D28 selected Diesel and D29 accepted this common direction;
revision-bound approval of the complete product Design remains pending.
Compare DB candidates against transaction ownership, native PostgreSQL ENUM and
SQLite mapping, cancellation, migration operations, and community sustainability
before choosing. Preserve this evidence without treating candidate use in a
probe as production selection authority.

## Database Candidate Execution Evidence

### Scope and setup

The second independent probe compiled SeaORM 2.0.4, SQLx 0.9.0, Diesel 2.3.14
and diesel-async 0.9.2 with rustc 1.99.1. SQLite used a bundled library through
the selected drivers; PostgreSQL 17.11 ran in a private disposable local cluster
with Unix-socket access and was stopped/removed afterward. No product database
or deployment was touched.

The same small membership relation used a native PostgreSQL ENUM and a constrained
SQLite TEXT role, with MANAGER/MEMBER as the finite domain. This demonstrates
that retaining PostgreSQL's enum enforcement need not mean forcing PostgreSQL
DDL onto SQLite. It does not approve this schema as an implemented migration.

Raw DDL prepared the fixtures. Ordinary reads/inserts were not all raw SQL:

| Candidate | Actual API exercised | Both backend outcomes |
| --- | --- | --- |
| SeaORM | DeriveEntityModel, DeriveActiveEnum, ActiveModel insert, entity find, TransactionOptions with SQLite Immediate | Typed role CRUD; invalid-role/duplicate rejection; explicit rollback; no inserted row after owned idle transaction dropped |
| SQLx | Bound SQL with typed reads and a derived native PostgreSQL enum; SQLite begin_with BEGIN IMMEDIATE | Constraints, explicit rollback and owned idle transaction drop behaved as expected |
| Diesel + diesel-async | Typed table/query DSL; custom PostgreSQL ToSql/FromSql enum; SQLite SyncConnectionWrapper; immediate_transaction and PostgreSQL transaction callbacks | Typed queries and ordinary-error rollback passed; separate native enum/SQLite type mappings were explicit |

All six candidate/backend runs completed successfully against their stated
assertions. The initial Diesel probe used deprecated root derive-macro paths
while disabling default features; switching to the documented defining modules
resolved compilation without a dependency override. This was probe-code setup,
not evidence that Diesel lacked the capability. Build, targeted Clippy with
warnings denied, and rustfmt checks passed.

### Cancellation finding

After a Diesel transaction inserted a row and signaled a barrier, its callback
waited indefinitely. Dropping the callback future left the uncommitted row
visible on the same connection on both engines. The probe then explicitly
rolled back and closed its private connections. This reproduces the documented
diesel-async transaction-cancellation warning; it does not show that the row was
committed or another connection could see it.

SeaORM and SQLx tests instead dropped the transaction owner after a completed
insert, and a subsequent same-connection read found no row. These tests cover an
idle transaction cancellation boundary, not cancellation during every in-flight
query/commit or every pool-reuse path. Do not claim universal cancellation safety.

The normal official-pool path below supplies connection retirement when the
cancelled repository operation owns its lease. The retained raw connection
experiment alone must not be treated as a defect or reason to reject Diesel.

### Follow-up: official pooled cancellation path

The diesel-async 0.9.2 bb8/deadpool adapters check `is_broken`, which includes
open non-test transactions. Runtime verification used bb8 0.9.1 and PostgreSQL
17.11 with pool size one. A repository-style future acquired its own lease,
inserted inside a transaction, signaled a barrier, then waited. Dropping the
whole future dropped the transaction and lease together.

Next checkout used a different PostgreSQL backend PID; the cancelled insert
was absent and a subsequent insert worked. There was no manual rollback or
custom pool/supervisor. Clippy with warnings denied passed for that probe.
Deadpool's equivalent path was source-inspected, not runtime-tested.

The same operation-owned cancellation pattern was subsequently checked on a
file-backed SQLite database with SyncConnectionWrapper and the official bb8
adapter, pool size one. After cancelling an immediate transaction, the next
connection did not retain a temporary-table identity marker, the persistent
table contained no cancelled insert, and a subsequent insert succeeded. This
verified connection replacement and rollback at that cancellation boundary.
Clippy with warnings denied and rustfmt checks passed for both pool probes.

This revises the earlier weighting of the raw cancellation observation.
Connection ownership and the documented pool contract are the relevant criteria.
Commit ambiguity and other cancellation points still need targeted checks.

### SQLx schema-checking verification

Additional feature-separated probe binaries used query! against each actual
schema at compile time. Both compiled, ran, and returned the expected row;
the PostgreSQL result included the declared native role enum. Replacing a
selected column with a nonexistent one caused compilation to fail on each
engine. Restoring the query and rebuilding/running succeeded.

This verifies the option to regain schema-aware checks in a SQL-first approach,
but each build was checked against its own engine. It is not one portable
query macro automatically validated against both. The first locked run correctly
refused a changed macro-feature lockfile; normal Cargo lock generation followed
by locked checks resolved it. Product CI would need migrated-schema/offline
metadata discipline and integration tests for dynamic queries.

### Recorded selection after the broader review

The requester selected **SQLite/PostgreSQL with Diesel** under D28 after
reviewing adoption, type safety, local DB alternatives, and the official pooled
cancellation path. Earlier SeaORM-first and SQLx-first recommendations were
research proposals, not accepted library decisions.

SQLx's cost is owned SQL and more explicit per-backend query/schema mapping.
SeaORM's value is reusable entities and ordinary CRUD, at the cost of another
abstraction whose generated statements and behavior must still be tested.
Both share the SQLx driver ecosystem; popularity is important evidence, not an
argument that SeaORM is unsupported.

The selection does not establish product concurrency, complete migrations,
performance, all pool-recovery paths, or final Design feasibility. Those remain
required implementation/verification work.

## Post-Selection Migration and RPC Checks

### Diesel migration harness

An independent probe used diesel_migrations 2.3.2 with embedded, backend-specific
SQL fixtures and the existing Diesel/diesel-async versions. The one-shot command
used SqliteConnection and the supported AsyncConnectionWrapper<AsyncPgConnection>;
it did not call blocking migration code from an async server request.

On both actual databases it verified:

1. First migration applies and records one history entry.
2. Running the same source again applies zero migrations.
3. A deliberately failing second migration creates no lasting partial table and
   records no successful history entry; the prior successful migration remains.
4. The migrated native PostgreSQL ENUM/constrained SQLite role and deferred
   owner-membership foreign key enforce the tested ownership invariant.

The fixture tested default transactional migrations, not every DDL statement
or nontransactional migration. A later failing migration does not undo prior
successful migrations. These files were research fixtures, not product schema.

An additional empty SQLite test demonstrated that has_pending_migration creates
the __diesel_schema_migrations table. Source inspection shows applied_migrations
calls database setup. These APIs must not be used as supposedly read-only
ordinary-server readiness checks under D15. Check existing schema metadata with
read-only queries instead; only the explicit migration command initializes it.
Concurrent migration locking and the actual complete migration set remain to be
verified. Clippy with warnings denied passed for the migration probe.

### Generated gRPC boundary

An isolated Tonic 0.14.8/Prost 0.14.4 probe generated both client and server code
from a small Protobuf source using protoc 36.2. It bound separate loopback
general/Admin listeners, injected a shared fixture domain service, and made
real client calls. The single integration test verified:

- Missing bearer credential returns UNAUTHENTICATED.
- One valid credential can call both endpoints while Admin authority is present.
- Changing the fixture's current Admin authority rejects subsequent Admin calls
  while general calls remain valid.
- Optional string omission differs from an explicitly empty value.
- Unknown enum numbers are rejected by explicit validation, not assumed safe
  merely because Protobuf generated a Rust field.
- Both owned servers complete graceful shutdown and are joined within a bound.

The credential and authority store were deliberately simple fixtures, not
product authentication. TLS, real sessions/DB authorization, TypeScript client
generation and full product schemas were not exercised. Locked tests, targeted
Clippy with warnings denied, and rustfmt checks passed.

### Failure source and stack reporting

A separate executable captured std::backtrace::Backtrace at a simulated storage
adapter boundary, propagated the typed error and its source through the service,
and emitted one structured tracing JSON event at the handling boundary.
With RUST_BACKTRACE=1 the captured stack included the adapter function; the
wire-facing gRPC INTERNAL message contained neither the underlying cause nor
the trace. The output parser verified one event, operation/source fields and
captured stack. Clippy/rustfmt checks passed.

This demonstrates explicit boundary-stack capture and one-time structured
reporting. It is not equivalent to recovering every original driver stack:
frames unwound before wrapping cannot be recreated. Production policy must
choose capture locations and protect credentials/payloads; the probe introduces
no direct telemetry SDK calls or new reporting service.

### Common stack direction accepted under D29

The selected DB layer is Diesel. The accepted common foundation is Tokio with
explicit task ownership, Tonic/Prost adapters, constructor injection and narrow
typed service/repository boundaries, thiserror-style domain errors with explicit
infrastructure reporting, tracing, and standard Rust compiler/Clippy/rustfmt gates.
The requester approved this direction after reviewing the probes. The evidence
establishes feasibility of representative boundaries, not permission
to implement or final approval of the entire product Design.
