# Build from the repository root. Every base is a verified Docker Official Image
# multi-architecture index; target platform selection is owned by BuildKit.
ARG RUST_IMAGE=rust:1.99.1-bookworm@sha256:9181b56c2a21ab1d5e73cffe04fc762426d30aa7fac3c91141dd033f49a7fc8b
ARG NODE_IMAGE=node:24.22.1-bookworm-slim@sha256:be8204d4bd327363357f532187b752ff62759c31bedf98e9e18fc08c9cbd605c
ARG RUNTIME_IMAGE=debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3

FROM ${RUST_IMAGE} AS backend-build
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config \
    && rm -rf /var/lib/apt/lists/*
COPY rust ./rust
COPY proto ./proto
WORKDIR /src/rust
RUN cargo build --locked --release --package chapchoo

FROM ${NODE_IMAGE} AS node-build-runtime

# Protocol generation is a real build prerequisite. Keep its Rust/vendored
# protoc toolchain and the matching Node package tooling in this build stage;
# neither toolchain is copied into a final runtime image.
FROM backend-build AS web-build-base
ARG TARGETARCH
COPY --from=node-build-runtime /usr/local/ /usr/local/
WORKDIR /src
# Build-only tooling. Nothing is installed on the deployment host or inherited
# by final web images except the matching Node binary and traced application.
RUN apt-get update && apt-get install -y --no-install-recommends python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && test "$(node --version)" = "v24.22.1" \
    && npm install --global pnpm@11.26.0
COPY typescript ./typescript
COPY scripts/generate-rpc-clients.sh ./scripts/generate-rpc-clients.sh
COPY scripts/package-linux.py ./scripts/package-linux.py
COPY images/chapchoo/stage-web.py ./images/chapchoo/stage-web.py
WORKDIR /src/typescript
ENV NEXT_TELEMETRY_DISABLED=1 TURBO_TELEMETRY_DISABLED=1
RUN pnpm install --frozen-lockfile
COPY --from=backend-build /src/rust/target/release/chapchoo /inputs/chapchoo

FROM web-build-base AS general-build
ARG TARGETARCH
ARG GENERAL_PUBLIC_URL=http://localhost:3003
RUN mkdir -p apps/chapchoo-web/public \
    && CHAPCHOO_PUBLIC_URL="$GENERAL_PUBLIC_URL" pnpm exec turbo run build --filter=@chapchoo/web \
    && python3 /src/images/chapchoo/stage-web.py --app general --architecture "$TARGETARCH" \
       --typescript-root /src/typescript --binary /inputs/chapchoo --node /usr/local/bin/node \
       --node-license /usr/local/LICENSE --output /out/general

FROM web-build-base AS admin-build
ARG TARGETARCH
ARG ADMIN_PUBLIC_URL=http://localhost:3004
RUN mkdir -p apps/chapchoo-admin-web/public \
    && CHAPCHOO_PUBLIC_URL="$ADMIN_PUBLIC_URL" pnpm exec turbo run build --filter=@chapchoo/admin-web \
    && python3 /src/images/chapchoo/stage-web.py --app admin --architecture "$TARGETARCH" \
       --typescript-root /src/typescript --binary /inputs/chapchoo --node /usr/local/bin/node \
       --node-license /usr/local/LICENSE --output /out/admin

FROM ${RUNTIME_IMAGE} AS runtime-base
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 chapchoo \
    && useradd --no-log-init --create-home --uid 10001 --gid 10001 --shell /usr/sbin/nologin chapchoo \
    && mkdir -p /opt/chapchoo && chown 10001:10001 /opt/chapchoo
WORKDIR /opt/chapchoo
COPY --chown=10001:10001 LICENSE ./LICENSE
USER 10001:10001
ENTRYPOINT ["/opt/chapchoo/chapchoo"]

# A backend image contains neither a Node runtime nor a web application.
FROM runtime-base AS backend
COPY --from=backend-build --chown=10001:10001 /src/rust/target/release/chapchoo ./chapchoo
EXPOSE 50051 50052
CMD ["serve", "--role", "both", "--general-listen", "0.0.0.0:50051", "--admin-listen", "0.0.0.0:50052"]

FROM runtime-base AS web-runtime-base
USER root
RUN apt-get update && apt-get install -y --no-install-recommends libstdc++6 \
    && rm -rf /var/lib/apt/lists/*
USER 10001:10001

FROM web-runtime-base AS general-web
COPY --from=general-build --chown=10001:10001 /out/general/ ./
EXPOSE 3003
CMD ["serve", "--role", "none", "--web", "general", "--general-web-listen", "0.0.0.0:3003"]

FROM web-runtime-base AS admin-web
COPY --from=admin-build --chown=10001:10001 /out/admin/ ./
EXPOSE 3004
CMD ["serve", "--role", "none", "--web", "admin", "--admin-web-listen", "0.0.0.0:3004"]
