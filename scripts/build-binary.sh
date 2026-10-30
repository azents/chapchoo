#!/usr/bin/env bash
# Mac and Linux use the same pinned Linux ABI baseline as CI.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
if [[ $# -ne 1 || "$1" != /* || -e "$1" ]]; then
  printf 'Usage: %s /absolute/new-artifact-output\n' "$0" >&2
  exit 2
fi
case "$(uname -m)" in
  x86_64) architecture=amd64 ;;
  aarch64|arm64) architecture=arm64 ;;
  *) printf 'Unsupported target architecture\n' >&2; exit 2 ;;
esac
cd "$root"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT
docker buildx build --platform "linux/$architecture" \
  -f images/chapchoo/binary-build.Dockerfile --target binary-artifacts \
  --output "type=tar,dest=$temporary/binary.tar" .
mkdir -p "$1"
tar -xf "$temporary/binary.tar" -C "$1"
