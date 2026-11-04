---
title: "Linux artifacts and image assembly"
---

# Linux artifacts and image assembly

Build owners produce deployment-neutral inputs; packaging validates and assembles those exact artifacts without rebuilding them.

## Assembly and images

The account acceptance Assembly uses separate public `serve --groups client,control`
and `serve --groups admin` processes and loopback RPC listeners. General/Admin web
listeners receive only their corresponding explicit backend URL, so a missing
Admin-only contract cannot be hidden by a combined union. Secondary-user seeding
uses the separate current Client/Admin endpoints. Its private SWC cache is owned by the fixture;
Next build receives tooling settings, not deployment backend/public configuration.

`scripts/package-linux.py` accepts explicit built inputs; it neither builds nor
downloads. It validates matching ELF64 architecture, pinned official Node24.22.1
inventory/hash/bytes, safe links and app selection. Credential/state paths and
escaping/broken/cyclic links fail. Archive and SHA-256 sidecar are deterministic
for the input/epoch and are published without overwriting an existing output.
The standard gzip stream uses fast compression rather than spending CI time on
the maximum compression level; archive contents and validation are unchanged.
The distribution includes the project and Node licenses. Linux x86-64 and ARM64
are the supported targets; the release binary uses the fixed Bookworm builder.
Deployment hosts need no Node, pnpm, Turbo or build toolchain.

`images/chapchoo/Dockerfile` has independent backend/general-web/admin-web targets
from digest-pinned multiarchitecture official images. Separate build jobs own
Rust/vendored-protoc and Next compilation. Image assembly only validates and
copies their outputs; final images do not contain build toolchains. Runtime UID/GID is10001. Backend has no Node/web;
web has the central executable, matching Node and selected standalone app, and
receives no backend database/key credentials. Runtime supports a read-only root
with an explicitly writable temporary directory. Database state is separately
operator-provided, never baked into an image.

The standard runtime stages `/workspace` owned by UID:GID 10001:10001. A true
initial fresh Docker volume can copy that directory's ownership; existing
original volumes use no-copy. The runtime never uses a privileged ownership
repair helper. Custom mounts/images must meet the same non-root write contract.

## Connections

- [Foreground process ownership and health](runtime-process.md) — The composition root owns admission, startup and shutdown of its selected backend and web processes. A successful cleanup submission is not completed shutdown.
- [Helm and Argo deployment boundaries](deployment.md) — The public chart composes workloads and explicit prerequisites; offline rendering does not prove a live rollout or substrate cleanup.
- [Snapshot publication and downstream handoff](snapshot-publication.md) — A Snapshot publication binds architecture artifacts, image digests and source/run identity. Downstream deployment owns its own activation and data prerequisites.
- [Verification layers and release evidence](acceptance.md) — Each test layer establishes a specific owned contract. Synthetic SDK bytes, real Engine acceptance, browser journeys and live cluster rollout are different evidence.
- [Runtime web configuration and failures](web-runtime.md) — Web deployment configuration is resolved at runtime; decoding and reporting preserve safe, truthful failure boundaries.

## References

- [`scripts/package-linux.py`](../../../../scripts/package-linux.py)
- [`scripts/build-images.sh`](../../../../scripts/build-images.sh)
- [`images/chapchoo/Dockerfile`](../../../../images/chapchoo/Dockerfile)
- [`.github/workflows/linux-package.yaml`](../../../../.github/workflows/linux-package.yaml)
