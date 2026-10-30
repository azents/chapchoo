# Packaging only: provide architecture-matched binary-artifacts and web-artifacts
# named contexts from the separate build jobs or scripts/build-images.sh.
ARG NODE_IMAGE=node:24.22.1-bookworm-slim@sha256:be8204d4bd327363357f532187b752ff62759c31bedf98e9e18fc08c9cbd605c
ARG RUNTIME_IMAGE=debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3

FROM registry.k8s.io/kubectl:v1.35.0@sha256:0bb95b2a450875fc8ceaea2f9987a99fe27c228846e2e00b93b65ebb0d59034e AS kubectl-runtime
FROM scratch AS binary-artifacts
FROM scratch AS web-artifacts
FROM ${NODE_IMAGE} AS web-stage-base
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY scripts/package-linux.py ./scripts/package-linux.py
COPY images/chapchoo/stage-web.py ./images/chapchoo/stage-web.py
COPY --from=binary-artifacts /chapchoo /inputs/chapchoo
COPY --from=binary-artifacts /architecture /inputs/binary-architecture
COPY --from=web-artifacts /architecture /inputs/web-architecture

FROM web-stage-base AS general-build
ARG TARGETARCH
COPY --from=web-artifacts /general/standalone /src/typescript/apps/chapchoo-web/.next/standalone
COPY --from=web-artifacts /general/static /src/typescript/apps/chapchoo-web/.next/static
COPY --from=web-artifacts /general/public /src/typescript/apps/chapchoo-web/public
RUN test "$(cat /inputs/web-architecture)" = "$TARGETARCH" \
    && test "$(cat /inputs/binary-architecture)" = "$TARGETARCH" \
    && python3 /src/images/chapchoo/stage-web.py --app general --architecture "$TARGETARCH" \
       --typescript-root /src/typescript --binary /inputs/chapchoo --node /usr/local/bin/node \
       --node-license /usr/local/LICENSE --output /out/general

FROM web-stage-base AS admin-build
ARG TARGETARCH
COPY --from=web-artifacts /admin/standalone /src/typescript/apps/chapchoo-admin-web/.next/standalone
COPY --from=web-artifacts /admin/static /src/typescript/apps/chapchoo-admin-web/.next/static
COPY --from=web-artifacts /admin/public /src/typescript/apps/chapchoo-admin-web/public
RUN test "$(cat /inputs/web-architecture)" = "$TARGETARCH" \
    && test "$(cat /inputs/binary-architecture)" = "$TARGETARCH" \
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

FROM runtime-base AS backend
COPY --from=binary-artifacts --chown=10001:10001 /chapchoo ./chapchoo
COPY --from=kubectl-runtime /bin/kubectl /usr/local/bin/kubectl
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
