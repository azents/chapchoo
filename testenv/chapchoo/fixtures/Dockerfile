# Explicit acceptance-only existing CLI; never contacts a model.
ARG RUNTIME_BASE=chapchoo-runtime-base:acceptance
FROM scratch AS binary-artifacts
FROM ${RUNTIME_BASE}
LABEL org.opencontainers.image.title="Chapchoo deterministic conversation fixture"
COPY --from=binary-artifacts --chown=10001:10001 /chapchoo /opt/chapchoo/chapchoo
COPY --chown=10001:10001 --chmod=0555 testenv/chapchoo/fixtures/codex-app-server.cjs /opt/chapchoo/codex-fixture
