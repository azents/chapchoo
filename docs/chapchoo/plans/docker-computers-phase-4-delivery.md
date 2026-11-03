---
title: "Docker Computers Phase 4 Image and Real Lifecycle Acceptance"
created: 2026-11-03
tags: [computer, docker, implementation, acceptance]
---

# Phase Execution Plan

- Phase: `4/4 standard image and real Docker acceptance`.
- Branch/base: `feat/docker-computers-4-delivery` →
  `feat/docker-computers-3-native-runtime` at `7815553`
  ([PR #51](https://github.com/azents/chapchoo/pull/51)).
- PR boundary: non-root image saved-directory ownership and one browser-free
  real Docker integration lifecycle through public RPC and actual Control/
  Provider/Connector, with truthful evidence and current Specs. Existing browser
  E2E remains unchanged. No Mac build/distribution work.
- Test-owner correction: the requester clarified that Docker resource validation
  does not belong in browser E2E. Reuse testenv prerequisites, but own the native
  scenario in `typescript/apps/integration/` and a separate required CI job that
  consumes the existing same-run binary artifact; do not duplicate the UI flow.
- Inputs: phase3's DB-free CLI/selected public SDK/once-only lifecycle;
  174 Rust tests in50 groups,31 Docker/local OpenSSH cases,67 live RPC checks per
  database, mandatory hooks and independent review. Three preceding PRs are open.
- Scope correction: the requester explicitly clarified on2026-11-03 that context
  respect and natural portability for a later Mac build do not authorize current
  Mac-only compiler/CI/package/distribution work. Requirements were corrected,
  ADR-D5 appended and Design revision7 narrows M14 accordingly. Root removed its
  over-scoped Mac workflow, archive helper/tests and related claims before review.
- Deliverables: standard image initial non-root saved ownership; actual Docker
  owner Create/Connector/marker/Stop/Start/original-volume data/Delete journey
  with unrelated native canaries preserved; explicit prerequisite failures and
  verified individually scoped cleanup. Preserve Kubernetes and General/Admin
  native labels, selected-context semantics and host portability.
- Non-goals: Mac builds/archives/distribution or a current Mac validation gate;
  install/start/restart/configure Docker/Desktop; change contexts; provision host
  services; production daemon writes; proxy/tunnel/host-network modes; guest Docker
  credentials; new product exec APIs/browser matrices; force/prune cleanup;
  merge/deploy or support claims from unexecuted configurations.
- Interfaces: unchanged approved lifecycle/RPC/private-owner boundaries;
  `docker-provider` runtime flags; existing binary artifact and native Assembly
  ownership; public Docker SDK fixture operations and bounded test-only marker
  exec. Deployment configuration never enters published images.
- Approved mechanisms: M9/M10/M11/M12/M14, consuming existing M1–M16 native safety.
- Authority: confirmed [Requirements](../requirements/docker-261103-server-computers.md)
  REQ-1–REQ-6; [accepted ADR](../adr/docker-261103-server-computers.md) D1–D6 with
  requester D5 scope correction; [Design](../design/docker-261103-server-computers.md)
  revision7 (unchanged revision6 mechanisms plus explicit M14 narrowing);
  current Docker/web/delivery Specs and scoped SDK/testenv conventions.
- Design delta: None relative to that explicit requester-corrected baseline.
- Removal obligations: replace standard image's missing saved ownership and
  universal Kubernetes-only fixture/coverage claims. Preserve Kubernetes image/
  fixture/UID behavior and owner/Admin privacy. Remove the unrequested Mac build,
  package and gate additions entirely; portability is not distribution authority.
- Absence verification: image/workflow contracts, catalog and focused source
  search; real non-root marker/successor no-copy; existing native DTO tests;
  no raw Engine routes, broad cleanup or credentials in guests/artifacts.

| Workstream | Owner | Paths | Output | Validation |
| --- | --- | --- | --- | --- |
| Image saved storage | `/root` | `images/chapchoo/`, existing image/workflow owner tests | UID:GID10001:10001 directory, no privileged helper or baked deployment setting | Offline contracts and actual non-root initial write/successor no-copy |
| Real native prerequisites | `/root` | `testenv/chapchoo/src/{assembly,config,docker}.ts`, package manifest/lock, pure tests | Public SDK, unique scoped resources/canaries, bounded test-only exec and verified cleanup; no scenarios/assertions in support package | Pure helper/config/export tests, type/lint, integration-owned assembly |
| Minimal real product journey | `/root` | `typescript/apps/integration/`, separate required Linux CI gate | One serial public-RPC Create/connected/marker/Stop/Start/data/Restart/Delete/canary journey; existing browser journeys unchanged | Required real Engine path fails on missing prerequisites, same-run binary/image, no web or browser |
| Current contract/evidence | `/root` | Owning Specs, DEVELOPMENT/README, plans and scope correction | Passed/failed/unavailable evidence without expanding platform delivery | Full affected root checks, spec review, exact reviewer, hooks and PR4 |

- Integration order: corrected scope recorded → image/support → native integration/CI
  integration → root checks → frozen complete diff review → root corrections/
  affected reruns → PR4 → entire-stack exact-head CI and automatic-review handling.
  Do not poll earlier CI before the planned stack exists.
- Independent reviewer: `/root/computer-reviewer`, same exact read-only one-way
  reviewer. Root owns edits, integration, all verification and corrections.
- Final validation: existing Python image/build/workflow contracts; global TS
  format/lint/typecheck/build and support/app tests; docs/diff checks; required
  single Linux real Engine integration and preserved browser journeys. Rust/RPC gates rerun if
  those interfaces change. Remote modes remain separately reported opt-in real
  evidence; no missing prerequisite counts as passed or silently skipped.
- Scope drift: no new product authority, lifecycle retry/reset, source of truth,
  DB access in Providers, host-network/proxy workaround or Docker installation.
  No Mac build/distribution added merely because context resolution is portable.
- Context checkpoint: source image/SDK support and browser-free native integration implemented.
  Ordinary Assemblies remain loopback; only explicit real-Docker fixtures expose
  their trusted ephemeral Control route to the existing bridge. Guest credentials
  stay scoped to Connector; bounded native evidence is reported only after a
  passed journey. Unknown effects fail cleanup, never force/prune/replay.
  Root has passed global TS quality/all8 build tasks, General13/Admin20/web-core67
  tests, testenv10 pure+5 export tests and2 E2E unit tests. Existing Kubernetes
  owner/operator E2E and25 source-only Python contracts pass after removing the
  over-scoped Mac work. Independent review found one P2 in strict Restart polling.
  Root added explicit read-only absent/created readiness and requires a different
  verified running successor with the same daemon/volume. Native/ownership/config
  failures still propagate; no mutation retry or absence-as-success. Pure tests
  cover old-running→old-exited→old-removing→absent→new-created→new-running,
  changed original identity and terminating successor rejection. Global quality/
  build/support gates passed again. The exact reviewer confirmed the original
  P2 and its removing-state remainder resolved with no remaining findings. Rust/wire/product UI contracts are unchanged
  from phase3, whose174 Rust/67-per-engine RPC evidence remains valid.
  The initial PR52 CI passed all four existing browser journeys but failed the
  wrongly placed Docker browser scenario on Provider selection timeout. That log
  does not establish the native startup cause. The corrected required
  `docker-integration` job consumes the same-run binary without web artifacts or
  another Cargo build, starts no web/browser, and reports safe startup classes.
  Root reran Rust format/tests/Clippy/boundaries, global TS format/lint/typecheck/
  all9 builds, testenv11 pure+5 export tests, remaining E2E unit test and25
  Python contracts successfully. Complete corrected-phase review found one P2:
  normal once-only retirement also uses UNCERTAIN. Root requires the same request's
  completed power/connectivity report and bounds all in-progress observations,
  including UNCERTAIN, without mutation replay. All3 integration unit tests,
  lint/typecheck/build pass after the fix; the same reviewer confirmed the P2
  resolved with no remaining findings. CI41670547386 at3f33f59 passed9 checks,
  including all four browser journeys; native Docker failed on Provider early exit.
  ANSI formatting consumed the diagnostic budget, so root strips it before bounds
  and emits short static Docker SDK category/HTTP status records without sources.
  A strict wire regression independently reproduced the custom transport's missing
  direct HTTP framing contract; root now supplies Host and origin-form URI while
  preserving the selected connection and path/query. Full Rust gates passed again.
  An automated review's canary Start-response-loss leak is corrected by exact
  positively-created ID/label inspection and observed normal stop/remove; no Start
  replay or forced product cleanup. Global TS checks/all9 builds and15 pure+5 export
  support tests pass. The same reviewer found no ownership/privacy/framing issue
  in this targeted material delta. CI41674479629 atd50f12a passed9 checks,
  including browser acceptance, and native Provider registration now succeeded.
  The native journey failed after Create/connected began, but cleanup's generic
  exception masked the primary failure. Root preserves primary plus stage-tagged
  sanitized cleanup errors, registers immediate fallback teardown, and exposes
  only the existing bounded/redacted owned Provider diagnostic tail. Worker
  failure records contain static SDK class/status, never native payload or source.
  Full Rust175 tests/Clippy/format/boundaries and global TS/support/integration
  checks pass. The same reviewer confirmed error preservation/privacy with no
  findings; real exact-head native acceptance remains unpassed.
  Current host has no Engine socket; real fixture startup explicitly fails for
  missing authorized socket/image before launching DB/product. Actual Linux
  acceptance awaits the required CI gate after PR4 creation. No current Mac gate,
  merge/deploy or premature snapshot implementation claim is introduced.
