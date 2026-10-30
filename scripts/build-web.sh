#!/usr/bin/env bash
# Build in Linux even on a Mac; export complete traced runtime inputs.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
if [[ $# -ne 1 ]]; then
  printf 'Usage: %s /absolute/new-artifact-output\n' "$0" >&2
  exit 2
fi
output="$1"
case "$output" in
  /*) ;;
  *) printf 'Artifact output must be absolute\n' >&2; exit 2 ;;
esac
case "$(uname -m)" in
  x86_64) architecture=amd64 ;;
  aarch64|arm64) architecture=arm64 ;;
  *) printf 'Unsupported web runtime architecture\n' >&2; exit 2 ;;
esac
test ! -e "$output"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT
cd "$root"
docker buildx build --platform "linux/$architecture" \
  -f images/chapchoo/web-build.Dockerfile --target web-artifacts \
  --output "type=tar,dest=$temporary/web.tar" .
mkdir -p "$output"
tar -xf "$temporary/web.tar" -C "$output"
