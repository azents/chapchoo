# Model-free native compatibility uses a real compatible Codex artifact.
# The operator supplies architecture-matched binary-artifacts/chapchoo and
# native-artifacts/codex; neither context contains authentication or saved state.
ARG RUNTIME_BASE=chapchoo-runtime-base:acceptance
FROM scratch AS binary-artifacts
FROM scratch AS native-artifacts
FROM scratch AS fixture-artifacts
FROM ${RUNTIME_BASE}
LABEL org.opencontainers.image.title="Chapchoo real native tools fixture"
COPY --from=binary-artifacts --chown=10001:10001 --chmod=0555 /chapchoo /opt/chapchoo/chapchoo
COPY --from=native-artifacts --chown=10001:10001 --chmod=0555 /codex /opt/chapchoo/native-tools-codex
COPY --from=fixture-artifacts --chown=10001:10001 --chmod=0555 /codex-app-server.cjs /opt/chapchoo/codex-fixture
