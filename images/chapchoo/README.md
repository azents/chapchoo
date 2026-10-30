# Linux distributions and independent images

This directory implements build-time assembly for the approved account surface.
The central executable owns foreground execution; the archive and images supply
its fixed runtime paths. These recipes are not evidence that an artifact was run,
a foreign architecture was tested natively, or a cluster was deployed.

## Inputs and architecture

- Rust toolchain: `rust/rust-toolchain.toml`, currently **1.99.1**; build with the
  committed Cargo lockfile and a target-native Linux compiler/runtime baseline.
- Node: **24.22.1**, official Linux `x64` or `arm64` archive. The assembler accepts
  `x86_64` / `aarch64` respectively, validates the actual SHA-256 against both
  the supplied official inventory and its reviewed pinned inventory, then checks
  the ELF architecture of Node, Chapchoo and all traced ELF/native addon files.
- General/Admin Next standalone trees must be built separately on the matching
  platform from the frozen pnpm workspace. Preserve their traced workspace root,
  including the explicit `server.mjs` custom entrypoint. Include each app's built
  `.next/static` and `public` trees; these retain CSS, chunks and the actual
  Geist font files staged from the locked package with their license.
- A package build consumes already built artifacts. It does not build, download,
  run an input executable, install host dependencies, adopt a database, generate
  a key, or use a host Node/pnpm/Turbo fallback.
- The build-time assembler requires Python 3.11+ with gzip/lzma support.
  Deployment hosts run the bundled executable and Node; they do not need Python.

### Verified upstream metadata

Verified from first-party public release/registry endpoints on **2026-10-30 KST**:

- [Node release inventory](https://nodejs.org/dist/v24.22.1/SHASUMS256.txt):

  | Archive | SHA-256 |
  | --- | --- |
  | `node-v24.22.1-linux-x64.tar.xz` | `6c172a9ab6718f7b801fb5a7bd486af2ee5e32758086a04b84fcce2aa77095e6` |
  | `node-v24.22.1-linux-arm64.tar.xz` | `dbabf7a1cbf947e6059ffad2c27b80041db97dea1e553b62afdee7f13fca336e` |

- Docker Official Image index digests, queried through
  `https://registry-1.docker.io/v2/library/{node,rust,debian}/manifests/<tag>`:

  | Version tag | Multi-platform index digest |
  | --- | --- |
  | `node:24.22.1-bookworm-slim` | `sha256:be8204d4bd327363357f532187b752ff62759c31bedf98e9e18fc08c9cbd605c` |
  | `rust:1.99.1-bookworm` | `sha256:9181b56c2a21ab1d5e73cffe04fc762426d30aa7fac3c91141dd033f49a7fc8b` |
  | `debian:bookworm-slim` | `sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3` |

  Each index includes `linux/amd64` and `linux/arm64/v8`. Docker selects the
  architecture-specific manifest from this pinned index. A base-image update
  requires reviewing its public version tag and digest together.

- [Node 24.22.1 platform requirements](https://github.com/nodejs/node/blob/v24.22.1/BUILDING.md)
  list Linux x64/arm64 kernel **>=4.18**, glibc **>=2.28**, and compatibility
  with libstdc++ **>=6.0.25 / GLIBCXX_3.4.25** for the official binaries.

**The Node minimum is not the distribution's complete ABI promise.** The Rust
executable and native addons retain the libc/symbol requirements of their actual
builders. For example, the Docker recipe builds/runs on Debian Bookworm
(glibc 2.36 baseline), whereas a native Ubuntu 24.04 Rust build can require its
newer glibc baseline. ELF machine checks do not prove symbol compatibility.
Record the actual compiler, libc, `readelf --version-info`, dynamic dependency
inspection, and successful native startup for every released architecture.
The final image installs CA certificates/libgcc and web images additionally
install libstdc++; additional native dependencies must be established by actual
artifact acceptance rather than inferred from the Node support table.

## Build and assemble an archive

Run source builds on each selected native Linux architecture. The example uses
deployment-neutral builds. Set public URL/path and endpoints only when running
an artifact; the build needs no deployment configuration.

```sh
(cd rust && cargo build --locked --release --package chapchoo)
pnpm install --frozen-lockfile && pnpm run build
```

Download the matching official archive and `SHASUMS256.txt` into a build-artifact
cache outside the source inputs. For example, use the exact URLs below for x64,
and replace only `x64` with `arm64` for ARM64:

- `https://nodejs.org/dist/v24.22.1/node-v24.22.1-linux-x64.tar.xz`
- `https://nodejs.org/dist/v24.22.1/SHASUMS256.txt`

Then, from the repository root:

```sh
python3 scripts/package-linux.py \
  --architecture x86_64 \
  --binary rust/target/release/chapchoo \
  --node-archive /build-artifacts/node-v24.22.1-linux-x64.tar.xz \
  --node-shasums /build-artifacts/SHASUMS256.txt \
  --general-standalone typescript/apps/web/.next/standalone \
  --general-static typescript/apps/web/.next/static \
  --general-public typescript/apps/web/public \
  --admin-standalone typescript/apps/admin-web/.next/standalone \
  --admin-static typescript/apps/admin-web/.next/static \
  --admin-public typescript/apps/admin-web/public \
  --output /build-artifacts/chapchoo-linux-x86_64.tar.gz
```

The public font directories are produced by each app's normal build command.
For ARM64 use `--architecture aarch64` and the official `linux-arm64` Node input.
`--source-date-epoch <integer>` optionally sets deterministic archive timestamps;
its default is zero. Archives are sorted, normalize ownership/permissions, and
have no build-host username or gzip filename. An existing output is never
replaced. A complete archive is published atomically and its `.sha256` sidecar
contains its SHA-256 and basename.

The extracted root is `chapchoo-linux-x86_64/` or `chapchoo-linux-aarch64/`:

```text
chapchoo
LICENSE
runtime/bin/node
runtime/LICENSE
web/general/typescript/apps/web/server.mjs
web/general/typescript/apps/web/.next/static/...
web/admin/typescript/apps/admin-web/server.mjs
web/admin/typescript/apps/admin-web/.next/static/...
```

The full traced trees, relative symlinks and optional public assets remain under
their respective `web/` roots. Only Node's required executable and upstream
license are extracted from its archive. The tool rejects traversal, special
archive files, escaping links, wrong architectures, broken traced links,
credential/database/test-output paths and unexpected standalone application
roots. It never extracts through archive symlink parents. This bounded path
policy is not a content-level secret scanner: only supply reviewed, secret-free
build outputs. No `.env`, installation data, root key, bootstrap proof, provider
credential, browser result or cache belongs in the input trees.

Operators run the extracted `chapchoo`, not a source-workspace command. A web-only
selection needs its app's explicit public URL/backend endpoint and matching
listener configuration; it does not receive a DB/root/bootstrap/provider secret:

```sh
./chapchoo serve --role none --web general
./chapchoo serve --role none --web admin
```

Local HTTP requires an actual loopback listener and loopback public URL. Remote
public origins require HTTPS at an explicitly configured reverse proxy/Ingress.
Backend schema preparation remains an explicit operator/deployment command with
its existing original-key and database prerequisites. Installing an archive or
starting a web role never creates/migrates a database or supplies those secrets.

## Build independent Docker targets

Build from the repository root with the repository `.dockerignore`. It must
exclude host `node_modules`, `.next`, Rust `target`, `.env*`, keys/credential
folders, database state, test output and caches before sending the context.
Build outputs are produced separately using normal locked Cargo/pnpm resolution.
The final image Dockerfile only validates and copies existing target outputs; it
never recompiles the applications. CI uses native web and Bookworm binary build
jobs and transfers their matching outputs within the same workflow run.

```sh
# Mac/Linux: build Linux outputs first, then package all three images.
bash scripts/build-images.sh

# Reuse outputs from an independent matching-target build:
export WEB_ARTIFACTS=/absolute/build-artifacts/web
export BINARY_ARTIFACTS=/absolute/build-artifacts/binary
docker buildx bake --allow "fs.read=$WEB_ARTIFACTS" --allow "fs.read=$BINARY_ARTIFACTS" \
  -f images/chapchoo/ci-bake.hcl
```

The local wrapper uses separate `web-build.Dockerfile` and
`binary-build.Dockerfile` Linux build environments before final packaging.
Apple Silicon defaults to `linux/arm64`; x86-64 defaults to `linux/amd64`.
Target-specific cache mounts retain pnpm/Cargo/Next build work without importing
Mac node_modules. This is not evidence of a macOS app build or cross-architecture
emulation acceptance. Final images use the same named-input contract as CI.

- `backend`: central binary only; no Node or web tree. Default selection is both
  plaintext internal RPC listeners, with distinct ports50051/50052.
- `general-web`: central binary, matching Node and only the general traced app;
  default `serve --role none --web general`, internal listener0.0.0.0:3003.
- `admin-web`: central binary, matching Node and only the Admin traced app;
  default `serve --role none --web admin`, internal listener0.0.0.0:3004.
- All final targets run as UID/GID **10001**. They contain no runtime DB/root/
  bootstrap/SMTP credential configuration. Web roles need explicit runtime
  `CHAPCHOO_GENERAL_WEB_URL` / `CHAPCHOO_ADMIN_WEB_URL` with HTTPS when listening
  beyond loopback. Configure backend URLs through the corresponding selected
  role variables. Kubernetes may override CMD with the same foreground args;
  it must preserve the explicit wildcard internal listener configuration.
- Public origins, mount paths and endpoints are runtime configuration. The same
  image ID can run at different paths without rebuilding. The chart references
  already-existing backend
  Secrets; web pods do not receive those Secret values.
- Each web process's GET `<runtime-prefix>/health` confirms only web-process
  responsiveness. It is not backend/schema/product-journey evidence. Keep
  backend listeners private and use the established gRPC health contract.

Use native builders/runners for release acceptance on both architectures.
A `--platform` build by itself may use existing emulation and is not native-run
proof. These recipes neither install privileged binfmt handlers nor configure
an emulation service. Capture architecture, libc/native dependencies, selected
role startup, real assets, owned shutdown, foreign-resource safety and the
three product journeys in root-owned acceptance evidence before release.

## Snapshot distribution channel

The `Snapshot` workflow builds and runs these same targets on native amd64 and
arm64 before pushing architecture staging images and combining the complete
three-component indexes. Snapshot uses the same deployment-neutral application
build and assembly as other distribution artifacts. Home selects its origins
and `/admin` at runtime, outside the public artifacts. Native acceptance runs the
same bytes under root and non-root settings. Release publication of the same
commit must promote these artifacts/digests, not rebuild a channel-specific image.

The publication artifact carries one immutable source SHA/run/attempt and each
index's registry digest. Home independently verifies the source run and complete
artifact before proposing chart/image pins. Set the source repository variable
`DOWNSTREAM_DEPLOY_REPOSITORY=hardtack/home` and a least-privilege
`DOWNSTREAM_DEPLOY_TOKEN` Secret to enable notifications. A missing notification
configuration is reported explicitly. Main workflow publication, credential
readiness, Home initial full sync and live acceptance are separate gates; none
is established merely by this source configuration.

## Focused validation

```sh
python3 -m unittest scripts.tests.test_package_linux -v
python3 scripts/package-linux.py --help
ruff check scripts/package-linux.py scripts/tests/test_package_linux.py images/chapchoo/stage-web.py
ruff format --check scripts/package-linux.py scripts/tests/test_package_linux.py images/chapchoo/stage-web.py
```

The unit suite uses synthetic ELF/header/archive fixtures; it replaces the pinned
checksum inventory only in test-process memory and never executes a fixture.
It proves assembly guards/layout/determinism, not native executable validity,
Docker startup, shared-library completeness, browser networking or deployment.
Actual artifact/image execution and live-cluster acceptance remain separately
owned and authorized integration gates.
