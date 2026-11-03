---
title: "Rust Convention Research, Cross-Language Disposition and Enforcement"
created: 2026-11-04
tags: [rust, conventions, research, quality]
document_role: supporting
document_type: supporting-research
---

# Rust Convention Research and Enforcement

## Scope and evidence

This report supports [rust-261104/REQ](../requirements/rust-261104-engineering-conventions.md).
It is research and enforcement evidence, not an additional convention index.
The small bodies in `.claude/conventions/rust/` carry human-review rules; compiler,
Rustfmt, Clippy and executable boundary checks carry mechanical requirements.

The setup PR starts independently from main `210db535`. Its Rust inventory is
105 files / 32,814 lines across the domain, crypto, persistence, protocol, service,
RPC, mail, Kubernetes and foreground app/test packages. The separately unmerged
Docker stack ends at `439c01e` with ten successful checks, including actual native
lifecycle acceptance. Its new Rust code remains in the subsequent adoption
inventory; this main-based PR does not import that feature's changes.

All 36 Python and 20 TypeScript convention bodies were read, not only their index
titles. All global bodies were checked for inheritance/duplication. The package
manifests, public crate surfaces, quality command, generated-code boundaries,
rule loader, representative authority/error/time/async boundaries and existing
lint suppressions were inspected. Whole-codebase adoption coverage is a separate
next phase; this survey is not a claim that every Rust function was refactored.

## Primary references and application

| Reference | Applied evidence | Limits of the recommendation |
| --- | --- | --- |
| [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html) | Systematic review of API, type, ownership, errors and teardown concerns | Not every library recommendation belongs in this private application workspace; naming, signatures and trait boilerplate are not human convention bodies. |
| [Type safety: C-NEWTYPE, C-CUSTOM-TYPE, C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html) | Distinct authority/units and genuine state alternatives carry semantic meaning | No wrapper for every primitive, builder for every constructor or enum for independent facts. |
| [Flexibility: C-CALLER-CONTROL, C-GENERIC, C-OBJECT](https://rust-lang.github.io/api-guidelines/flexibility.html) | Ownership transfer, real substitution boundaries and caller requirements | No trait/dynamic-dispatch layer solely to replace a concrete collaborator in a test; no generic abstraction without an actual consumer. |
| [Interoperability: C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html) | Typed recoverable errors and retained causal information | Broad Serialize/Default/Debug recommendations cannot override credential privacy or grant validated construction authority. |
| [Dependability: C-VALIDATE, C-DTOR-FAIL/BLOCK](https://rust-lang.github.io/api-guidelines/dependability.html) | Validated value construction and explicit fallible/async teardown | Existing panic/unsafe lint policy is stricter than some guideline examples; no copied panic examples or unchecked escape APIs. |
| [The Rust Book: recoverable errors and validated types](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html) | Result is normal failure transport, not a disguised successful response; validate at an owning boundary | The repository already forbids explicit panic/unwrap/expect through tooling; that syntax policy is not repeated as prose. |
| [The Rust Book: public re-exports](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#re-exporting-names-with-pub-use) | A crate facade may deliberately differ from its internal module structure | Python/TypeScript barrel-file bans are not Rust API design. |
| [Tokio select cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety) | Selection drops losing branch futures; caller cancellation is not proof of physical effect cancellation | Evaluate the operation and every await boundary; replacing select with another race combinator alone does not establish effect safety. |
| [Tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown) | Stop admission, signal owners and await completion | Bounded shutdown still distinguishes unfinished effects from completed cleanup. |
| [Tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) | Started blocking work cannot be aborted; capacity and inputs need a real owner | A timeout or dropped JoinHandle must not release the permit while physical work continues. |
| [Tokio shared state](https://tokio.rs/tokio/tutorial/shared-state) | Choose short synchronous critical sections versus owned async-resource sequencing from the protected invariant | No blanket Tokio mutex, actor or sharded-map mandate. Compiler/linter examples are verified against the pinned toolchain, not copied from old tutorial diagnostics. |
| [Tracing instrument](https://docs.rs/tracing/latest/tracing/attr.instrument.html) | Default argument and return/error recording can cross disclosure boundaries; async context needs intentional scope | Never assume a dependency's Debug/Display or an instrument attribute is credential-safe. |
| [Cargo workspace lint inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table) | Central workspace policy with explicit member inheritance | Policy must be exercised through the actual quality command, not only declared. |
| [Clippy configuration](https://doc.rust-lang.org/clippy/configuration.html) and [lint catalogue](https://rust-lang.github.io/rust-clippy/stable/index.html) | Use supported individual lints and measured baseline impact | Pedantic is explicitly aggressive; a blanket group is not evidence of an appropriate project policy. |

Public sources were fetched directly when the existing document-reader connection
returned `Session not found`. The relevant primary-source text was then read;
failed tool calls were not counted as research evidence.

## Complete Python disposition (36 / 36)

Paths below are relative to `.claude/conventions/python/`. **Adapt** means semantic
intent survives with Rust-specific meaning; **Tools** means no duplicate manual
rule; **Inherited** means the global body already owns the concern; **Exclude**
means the source's framework/API or policy is not applicable.

| Source file | Disposition | Rust rationale / destination |
| --- | --- | --- |
| alembic-revision-only | Adapt + exclude syntax | Current Diesel SQL migrations, not Alembic commands; retain paired-engine history/data obligations in durable evolution. |
| apps-depend-on-libs-only | Tools | Existing Cargo metadata dependency-boundary check owns app-leaf direction and approved dependencies. |
| avoid-underscore-dependency-fields | Adapt + exclude naming | Rust private fields are idiomatic; concrete versus trait collaborators depends on a real boundary, not Python underscore style. |
| bind-extra-for-shared-log-context | Adapt | Safe operation correlation follows async work; tracing span semantics differ from Python logger binding. |
| boolean-no-is-prefix | Exclude | Rust API idioms differ; naming is a tool concern, not a copied manual rule. |
| cancellederror-separately | Adapt | Dropped futures, task ownership and unresolved side effects replace Python exception-branch syntax. |
| catch-narrowest-exception | Adapt | Preserve known absence/domain rejection versus transport, decoding and unknown-effect failures in typed outcomes. |
| dedent-llm-prompts | Exclude | Python string mechanics and a currently absent Rust prompt subsystem do not justify a speculative rule. |
| dependency-injection-no-singletons | Adapt | Composition owns concrete collaborators; operations receive the authority they actually need. |
| dependency-injection-over-post-init | Adapt | Fold into composition ownership; no Python dataclass/post-init emulation or gratuitous trait layer. |
| dependency-version-pin | Tools + inherited | Cargo workspace dependencies, Cargo.lock and --locked already own resolution; global no-override policy still applies. No Python version-specifier scheme. |
| engine-changes-all-lowerers | Adapt | Shared contracts require all supported implementations, currently including both DB engines and native adapters; do not invent Rust LLM lowerers. |
| exhaustive-union-match | Tools | Compiler exhaustiveness and lint diagnostics own syntactic completeness; semantic state selection is a separate human decision. |
| explicit-named-indexes | Exclude syntax + adapt meaning | SQLAlchemy model syntax and aliases do not transfer; existing SQL migration/index identity must stay coherent across supported engines. Mechanical naming checks are not prose rules. |
| import-from-defining-module | Exclude ban | Rust pub use is an intentional crate facade; expose a consumer contract rather than implementation handles. |
| layered-architecture-no-skip | Adapt + tools | Cargo graph checks own import direction; completed repository operations and transaction authority still need semantic review. |
| linear-migration-chain | Adapt | Preserve already-applied SQL history and paired-engine acceptance; no Alembic merge-revision command or new migration scheme. |
| llm-tool-schema-compatibility | Exclude | No current Rust LLM function-tool schema subsystem; existing typed ingress/global SDK rules remain applicable. |
| log-errors-once | Adapt | One boundary owns incident-level reporting; lower layers preserve typed outcomes rather than duplicate incidents. |
| logger-exception-for-errors | Adapt | Keep causal errors privately and publish only safe context; dependency Display/Debug is not automatically safe, and Python logging APIs are not prescribed. |
| never-discard-exceptions | Adapt | Result/Option carry different contracts; unknown failures must not become absence/success, while expected rejection stays a legitimate typed result. |
| no-empty-value-defaults | Adapt | Missing, explicit empty, unresolved and valid default are separate semantic choices; do not ban meaningful empty collections or Default universally. |
| no-explicit-5xx-http-exception | Adapt + exclude status ban | Tonic statuses are not FastAPI HTTP exceptions. Translate expected domain errors intentionally; retain internal causes without exposing them or pretending failure was success. |
| no-getattr-setattr-type-bypass | Tools + inherited | Compiler typing and unsafe policy cover Rust mechanics; global typed ingress owns raw-payload conversion. No Python dynamic-field rule. |
| postgres-enum-types | Exclude | PostgreSQL-only native ENUM policy contradicts the current SQLite/PostgreSQL contract; preserve the existing verified representation. |
| rdb-index-constants | Exclude | SQLAlchemy class-constant layout is not the Diesel SQL schema model. |
| required-new-fields | Adapt | Compiler already requires struct fields; review semantic defaults that would hide an unaudited authority/migration/configuration choice. |
| sphinx-style-docstrings | Exclude syntax + tools | Rustdoc is not Sphinx; formatting/heading presence can be linted. Human review concerns actual boundary semantics, not prescribed comment syntax. |
| structured-logging-extra | Adapt | Safe searchable correlation and disclosure ownership matter; do not prescribe Python extra/f-string APIs or duplicate stdout/debug lints. |
| structured-multi-field-returns | Adapt | Named state/outcome data is useful where meaning or authority can be swapped; ordinary Rust tuples remain valid for naturally paired local values. |
| timezone-aware-datetimes | Adapt | Distinguish durable acceptance time from process-relative elapsed time and identify units; no Python datetime API. Numeric conversion mechanics belong to lints. |
| toolkit-prompt-placement | Exclude | No current Rust Toolkit prompt machinery; no new subsystem is implied. |
| transaction-db-operations-only | Adapt | Repository-owned, completed DB-only transactions; external effects need explicit authority revalidation around them. |
| transparent-error-handling | Adapt | Rust Result is correct recoverable error transport, not an HTTP200 error envelope; distinguish expected, unexpected and physically uncertain outcomes. |
| type-safety-order | Tools + inherited | Compiler/lints and global typed ingress own mechanics; no cast/ignore syntax rule is copied into Rust bodies. |
| typeddict-for-update-apis | Inherited + adapt representation | Global omission/null/value contract remains authoritative; use the actual typed Rust boundary rather than Python TypedDict or a new clearing meaning. |

## Complete TypeScript disposition (20 / 20)

Paths below are relative to `.claude/conventions/typescript/`.

| Source file | Disposition | Rust rationale / destination |
| --- | --- | --- |
| adt-state-types | Adapt | Genuine alternatives and legal next actions carry state-specific data; independent observations remain independent. React loading/SSR state shapes do not transfer. |
| apps-depend-on-packages-only | Tools | Existing Cargo boundary checker owns dependency direction; no duplicate language rule. |
| avoid-type-assertions | Tools | Rust numeric as is not a TypeScript assertion. Checked-conversion lints own lossy casts; do not ban all as expressions or trait coercions. |
| browser-persistence-hooks | Exclude | Mantine/browser APIs have no Rust runtime analogue requiring a convention. |
| container-component-pattern | Exclude UI layout + adapt boundary intent | Composition versus domain/adapter authority is retained without React hooks/components/pages. |
| cookie-based-locale | Exclude | Existing product web locale contract is not a Rust language convention. |
| explicit-return-types | Tools | Rust function signatures already require return types; no manual rule. |
| mantine-tokens-no-hardcoded | Exclude | Visual tokens and CSS values are a web/design-system concern. |
| natural-technical-term-localization | Exclude UI scope | Existing localized UI remains governed by its web rule; operator privacy/error semantics are separate. |
| no-direct-process-env | Adapt + tools boundary | Human concern is a coherent validated runtime snapshot and no retargeting/fallback, not spelling an environment accessor in a designated filename. |
| no-edit-generated | Tools + inherited authority | Proto/build generation and generated-diff checks own current clients; do not copy obsolete OpenAPI paths or a second generated-code ban. |
| no-index-files | Exclude | Rust lib.rs, mod.rs and pub use are not TypeScript barrels and may define a deliberate facade. |
| no-reverse-layer-imports | Tools + adapt semantics | Cargo graph checks enforce direction; public consumer authority and completed persistence operations require semantic review. |
| page-tsx-thin-entrypoint | Exclude framework syntax + adapt composition | Rust composition roots wire execution and owners; no Next.js page/component layout. |
| pure-ui-stories | Exclude + inherited tests | Storybook is UI-owned. Global owned-contract/deterministic-test rules already apply to Rust tests. |
| server-component-by-default | Exclude | No Rust client/server-component directive analogue. |
| storybook-provider-structure | Exclude | No Rust Storybook/Mantine provider layer. |
| trpc-cache-invalidation | Exclude | No Rust tRPC query cache; native current-source authority is not a frontend invalidation API. |
| use-generated-public-client | Inherited + actual generator | Current Rust consumes generated tonic protocol; global supported-SDK policy and generation ownership already apply. No obsolete REST client calls. |
| user-relevant-status-copy | Exclude UI scope + adapt disclosure | Keep normal UI copy in web rules; typed Rust diagnostics/public errors must preserve privacy and actionable outcome meaning. |

## Rust-specific gaps covered by human review

The 16 bodies have these traceable concerns. The right column identifies the
engineering decision that a compiler/linter cannot infer from syntax alone.

| Rust body | Principal source / gap | Human decision, not a lint obligation |
| --- | --- | --- |
| invariant-bearing-values | C-NEWTYPE; Python required/empty-value concerns | Which values carry different authority, validation or units, rather than a wrapper for every primitive. |
| semantic-state-alternatives | TypeScript ADTs; C-CUSTOM-TYPE | Which combinations are impossible and permit different next actions; not boolean count or match exhaustiveness. |
| consumer-owned-api-boundaries | Python import rule / TypeScript barrels, adapted through Rust pub use | Which consumer contract and resource lifetime should be exposed, not visibility spelling. |
| composition-owned-collaborators | Both Python DI rules; C-GENERIC | Which owner supplies authority and which substitution is actually needed, not a trait for every constructor. |
| outcome-and-error-meaning | Narrow handling, transparent/undiscarded errors; C-GOOD-ERR | Known absence versus expected rejection versus unavailable/unknown effects, not merely using Result. |
| error-disclosure-ownership | Python logging family; tracing defaults | Which fields/cause may be disclosed and who owns incident reporting, not logger/macro syntax. |
| async-effect-ownership | Python cancellation; Tokio select | Whether cancellation is safe for a physical effect and who retains unresolved work, not a forgotten-future lint. |
| blocking-work-ownership | Tokio spawn_blocking | Where capacity must remain while a started closure survives caller timeout. |
| fallible-resource-shutdown | C-DTOR-FAIL/BLOCK; cleanup failure gaps | Which cleanup is completion evidence and how independent primary/cleanup failures remain observable. |
| invariant-scoped-synchronization | Tokio shared state | Which logical operation must be serialized; known guard-across-await mechanics are separately linted. |
| bounded-admission | Tokio ownership/shutdown; capacity gap | The actual acceptance/ACK point, resource budget and overload meaning, not a universal channel-type ban. |
| completed-persistence-operations | Python layering and database-only transactions | Which DB work must commit together and what authority to revalidate around external effects. |
| backend-contract-parity | Python all-lowerers concern | Which supported implementations share a changed semantic contract and which differences are native translation. |
| durable-evolution | Python migration/index concerns, adapted to actual SQL engines | Applied history, preserved data/dependencies and cross-engine equivalence, not migration filename syntax. |
| runtime-configuration-snapshots | TypeScript environment access / global runtime configuration | Coherent source, security policy and target lifetime, not access through a prescribed filename. |
| time-authority-and-units | Python time rule; current acceptance-time boundary | Which clock/instant authorizes the operation and its units, not numeric cast spelling. |

Existing global rules continue to own supported SDKs, typed raw ingress,
omission/null semantics, test ownership, deterministic synchronization, runtime-only
configuration, English text and dependency resolution. They are not duplicated.
The API checklist's universal naming/trait/doc syntax, generic builder and broad
serialization suggestions are not imported as dogma.

## Empirical tooling disposition

On the pinned Rust1.99.1 toolchain, an exploratory all-targets Clippy run with
pedantic + nursery produced 644 unique diagnostics / 40 kinds in handwritten
sources after excluding generated target output and duplicate target reports.
This is an inventory, not a failing mandatory policy or a justification for
copying those 40 lint rules into prose. Large clusters included 250 missing
Errors headings, 74 must-use suggestions and 66 visibility-style suggestions.
A missing-fields-in-Debug diagnostic described intentional credential omission.
Blanket adoption would mix safety, style, API changes and generated code.

The focused policy retains existing compiler/Rustfmt/default-Clippy checks and
explicit unsafe/debug/unwrap/expect/panic/stdout/stderr gates. It adds explicit
unused-must-use enforcement, eleven selected Clippy lints for known suspended
guards, forgotten futures, resource leaks, hidden exit and checked numeric
conversions, plus a real rustdoc broken-link gate. `rust/clippy.toml` identifies
tracing's entered-span guards for Clippy's supported invalid-await-type check.
These mechanical rules are intentionally absent from the convention bodies.
The focused probe found only four handwritten conversion sites (three production,
one test); existing validation already bounds their supported inputs. Setup makes
the minimal checked conversions needed to enforce the lint, with no product
contract change. Broad semantic adoption remains in later code PRs.

`allow_attributes_without_reason` was also measured: 242 tool-generated allowance
sites versus one handwritten test allowance. Generated source is not hand-edited,
and an additional broad generated-module exemption is not introduced merely to
adopt that style restriction. Reason presence is not duplicated as a human rule.
Likewise, const/style/length/trait boilerplate suggestions are not universal
conventions; compiler and existing default lints remain active.

## Setup and adoption delivery

The setup PR is based on main and contains the concise title-frontmatter
convention bodies, generated `rust/**` index, existing-loader regression checks
and focused Rustfmt/compiler/Clippy/rustdoc enforcement. These are implementation
choices for the Requirements' semantic-guidance, discoverability and automatic
enforcement outcomes, not additional product requirements.

Whole-codebase adoption follows in independently reviewable code PRs. Record
actual source coverage and owned-contract behavior tests; keep the unmerged
Docker stack explicit rather than importing its feature diff into setup.
Run mandatory local hooks, inspect each PR's exact-head CI and address grounded
review feedback. Do not mark the broader adoption complete when setup alone is
verified. This sequence preserves the requester's separate-delivery constraint.

## Initial integrated verification

Root executed the canonical gate with disposable PostgreSQL after the setup:
Rustfmt, all-target Clippy, two policy tests, all nine crates' private/public
rustdoc, 115 Rust tests in43 groups and the existing dependency-boundary check
passed. The compile-only policy test copies the actual workspace policy and
Clippy configuration into an isolated member: seven representative errors must
be denied, while checked conversion and properly instrumented async context pass.
The fixture is never executed and resolves cached dependencies offline.

All ten rule-loader tests passed, including the actual generated Rust index for
existing/nested/new paths and non-Rust negative cases, and every body remains
discoverable through its index title/link. The index was generated through
pre-commit's isolated environment, not by manually running the generator.
Ruff, documentation catalogue and Skill validation passed. Public Node/Rust RPC,
mandatory hooks, independent review and exact-head CI are separate delivery
evidence; they are not inferred from the configuration shown here.
