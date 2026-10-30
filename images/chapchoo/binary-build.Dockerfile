# Local Linux target build; final image packaging never compiles sources.
FROM rust:1.99.1-bookworm@sha256:9181b56c2a21ab1d5e73cffe04fc762426d30aa7fac3c91141dd033f49a7fc8b AS build
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY rust ./rust
COPY proto ./proto
WORKDIR /src/rust
RUN --mount=type=cache,id=chapchoo-release-${TARGETARCH},target=/src/rust/target \
    cargo build --locked --release --package chapchoo --timings \
    && mkdir -p /out/timings && cp target/release/chapchoo /out/ \
    && cp target/cargo-timings/*.html /out/timings/ \
    && printf '%s\n' "$TARGETARCH" > /out/architecture

FROM scratch AS binary-artifacts
COPY --from=build /out/ /
