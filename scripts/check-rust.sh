#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../rust"
expected="$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)"
actual="$(rustc --version | cut -d ' ' -f 2)"
if [[ "$actual" != "$expected" ]]; then
  printf 'Rust toolchain mismatch: expected %s, found %s\n' "$expected" "$actual" >&2
  exit 1
fi
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
python3 ../scripts/run-rust-tests.py
cargo metadata --locked --format-version 1 --no-deps | python3 -c '
import json, sys
packages = {p["name"]: p for p in json.load(sys.stdin)["packages"]}
allowed = {
    "chapchoo-domain": {"email_address", "thiserror", "zeroize"},
    "chapchoo-crypto": {
        "base64", "bcrypt", "chacha20poly1305", "chapchoo-domain", "getrandom", "hkdf",
        "hmac", "sha2", "subtle", "thiserror", "zeroize",
    },
    "chapchoo-persistence": {
        "chapchoo-crypto", "chapchoo-domain", "diesel", "diesel-async",
        "diesel_migrations", "libsqlite3-sys", "thiserror", "zeroize", "tokio",
        "base64", "tempfile",
    },
    "chapchoo-protocol": {
        "prost", "prost-types", "tonic", "tonic-prost", "tonic-prost-build",
        "protoc-bin-vendored",
    },
    "chapchoo-service": {
        "chapchoo-crypto", "chapchoo-domain", "chapchoo-persistence", "tokio",
        "zeroize", "url", "base64", "tempfile",
    },
    "chapchoo-rpc": {
        "chapchoo-crypto", "chapchoo-domain", "chapchoo-protocol", "chapchoo-service",
        "prost-types", "tonic", "tracing", "zeroize", "chapchoo-persistence",
        "base64", "tempfile", "tokio", "tokio-stream",
    },
    "chapchoo-mail": {
        "chapchoo-service", "aws-config", "aws-sdk-sesv2", "tokio", "chapchoo-domain",
    },
    "chapchoo": {
        "chapchoo-crypto", "chapchoo-domain", "chapchoo-persistence", "chapchoo-service",
        "chapchoo-rpc", "chapchoo-mail", "clap", "tokio", "tokio-stream", "tokio-util",
        "tonic", "tonic-health", "tracing", "tracing-subscriber", "thiserror", "zeroize",
        "url", "base64", "tempfile", "chapchoo-protocol",
    },
}
for name, dependencies in allowed.items():
    actual = {d["name"] for d in packages[name]["dependencies"]}
    if actual != dependencies:
        raise SystemExit(f"Unreviewed dependency boundary for {name}: {actual}")
if set(packages) != set(allowed):
    raise SystemExit("Workspace crate boundary must be reviewed explicitly")
for name, package in packages.items():
    runtime = {d["name"] for d in package["dependencies"] if d["kind"] != "dev"}
    if name != "chapchoo" and "chapchoo" in runtime:
        raise SystemExit("Reusable crates must not depend on the deployable app")
    if name in {"chapchoo-protocol", "chapchoo-rpc"} and "chapchoo-persistence" in runtime:
        raise SystemExit("Protocol runtime must not import the database implementation")
print("Rust dependency boundaries verified")
'
