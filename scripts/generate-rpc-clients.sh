#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
mode="${1:-}"
if [[ -n "$mode" && "$mode" != "--check" ]]; then
  printf 'Usage: %s [--check]\n' "$0" >&2
  exit 2
fi
generated="$root/typescript/packages/chapchoo-rpc-client/src/generated"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT
cd "$root/rust"
cargo run --locked --quiet \
  --manifest-path "$root/rust/Cargo.toml" \
  --package chapchoo-protocol --example generate_node -- "$temporary"
if [[ "$mode" == "--check" ]]; then
  diff --recursive --unified "$generated" "$temporary"
else
  mkdir -p "$generated"
  # Only generated files belong here. Replacing the directory also removes
  # obsolete output when messages or imports disappear from the source contract.
  rm -rf "$generated"
  mv "$temporary" "$generated"
fi
