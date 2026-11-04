---
title: "Snapshot publication and downstream handoff"
---

# Snapshot publication and downstream handoff

A Snapshot publication binds architecture artifacts, image digests and source/run identity. Downstream deployment owns its own activation and data prerequisites.

## Snapshot publication and Home handoff

The `Snapshot` workflow follows the requested Azents-style image publication:
main pushes or explicit main dispatch build on native x86-64/ARM64 runners.
Each runner builds independent backend/General/Admin images, exercises their
nonroot/read-only/private-RPC/static/shutdown boundary and only then pushes
architecture staging images. All builds are deployment-neutral; test execution
selects root and non-root paths at runtime. Snapshot is a development distribution
channel, not a Home build variant. A release of the same commit must promote
the same verified artifact/digest rather than rebuild an application variant.

After both native jobs succeed, publication validates exactly six matching
component/architecture metadata files, creates three multiarchitecture indexes
and checks their registry digest and exact Linux platform set. Immutable tags
include source SHA, run and attempt; only a complete set gets a
`snapshot-payload` artifact. An optional downstream dispatch carries only run
identity. Operators configure `DOWNSTREAM_DEPLOY_REPOSITORY` and
`DOWNSTREAM_DEPLOY_TOKEN`; missing downstream configuration is reported without
pretending a Home update occurred.

Home owns PostgreSQL/PVC/ExternalSecret prerequisites and private Traefik Gateway
routes separately from this chart. Its updater verifies the successful canonical
main Snapshot and downloads that run's artifact before atomically pinning chart
SHA and all images. It does not trust image data in a dispatch notification or
infer that Secrets are ready. Initial activation and live acceptance remain
explicit; no new nginx controller or public plaintext RPC is introduced.

## Connections

- [Linux artifacts and image assembly](account-delivery.md) — Build owners produce deployment-neutral inputs; packaging validates and assembles those exact artifacts without rebuilding them.
- [Helm and Argo deployment boundaries](deployment.md) — The public chart composes workloads and explicit prerequisites; offline rendering does not prove a live rollout or substrate cleanup.
- [Verification layers and release evidence](acceptance.md) — Each test layer establishes a specific owned contract. Synthetic SDK bytes, real Engine acceptance, browser journeys and live cluster rollout are different evidence.

## References

- [`.github/workflows/snapshot.yaml`](../../../../.github/workflows/snapshot.yaml)
- [`scripts/publish-snapshot.py`](../../../../scripts/publish-snapshot.py)
- [`scripts/tests/test_snapshot.py`](../../../../scripts/tests/test_snapshot.py)
