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
cargo test --locked --workspace
cargo metadata --locked --format-version 1 --no-deps | python3 -c '
import json, sys
packages = {p["name"]: p for p in json.load(sys.stdin)["packages"]}
allowed = {
    "chapchoo-domain": {"email_address", "thiserror", "zeroize"},
    "chapchoo-crypto": {
        "base64", "bcrypt", "chacha20poly1305", "chapchoo-domain", "getrandom", "hkdf",
        "hmac", "sha2", "subtle", "thiserror", "zeroize",
    },
}
for name, dependencies in allowed.items():
    actual = {d["name"] for d in packages[name]["dependencies"]}
    if actual != dependencies:
        raise SystemExit(f"Unreviewed dependency boundary for {name}: {actual}")
print("Rust dependency boundaries verified")
'
