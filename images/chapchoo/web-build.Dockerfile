# Dedicated Linux web builder for local Mac/Linux orchestration.
# Final image assembly consumes its exported output, never rerunning Next.
ARG RUST_IMAGE=rust:1.99.1-bookworm@sha256:9181b56c2a21ab1d5e73cffe04fc762426d30aa7fac3c91141dd033f49a7fc8b
ARG NODE_IMAGE=node:24.22.1-bookworm-slim@sha256:be8204d4bd327363357f532187b752ff62759c31bedf98e9e18fc08c9cbd605c

FROM ${NODE_IMAGE} AS node-runtime
FROM ${RUST_IMAGE} AS build
ARG TARGETARCH
COPY --from=node-runtime /usr/local/ /usr/local/
RUN apt-get update && apt-get install -y --no-install-recommends python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && npm install --global pnpm@11.26.0
WORKDIR /src
COPY rust ./rust
COPY proto ./proto
COPY typescript ./typescript
COPY scripts/generate-rpc-clients.sh scripts/prepare-web-artifacts.py scripts/package-linux.py ./scripts/
ENV NEXT_TELEMETRY_DISABLED=1 TURBO_TELEMETRY_DISABLED=1
WORKDIR /src/typescript
RUN --mount=type=cache,id=chapchoo-web-cargo-${TARGETARCH},target=/src/rust/target \
    --mount=type=cache,id=chapchoo-web-pnpm-${TARGETARCH},target=/pnpm/store \
    --mount=type=cache,id=chapchoo-general-next-${TARGETARCH},target=/src/typescript/apps/chapchoo-web/.next/cache \
    --mount=type=cache,id=chapchoo-admin-next-${TARGETARCH},target=/src/typescript/apps/chapchoo-admin-web/.next/cache \
    pnpm install --frozen-lockfile --store-dir /pnpm/store \
    && pnpm exec turbo run build --filter=@chapchoo/web --filter=@chapchoo/admin-web \
    && case "$TARGETARCH" in amd64) machine=x86_64 ;; arm64) machine=aarch64 ;; *) exit 1 ;; esac \
    && python3 /src/scripts/prepare-web-artifacts.py \
       --typescript-root /src/typescript --output /out/web --architecture "$machine"

FROM scratch AS web-artifacts
COPY --from=build /out/web/ /
