#!/usr/bin/env bash
# Local orchestration uses exactly the web-output contract consumed by CI.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT
bash "$root/scripts/build-web.sh" "$temporary/web"
bash "$root/scripts/build-binary.sh" "$temporary/binary"
cd "$root"
export WEB_ARTIFACTS="$temporary/web"
export BINARY_ARTIFACTS="$temporary/binary"
docker buildx bake --allow "fs.read=$WEB_ARTIFACTS" --allow "fs.read=$BINARY_ARTIFACTS" \
  -f images/chapchoo/ci-bake.hcl backend general-web admin-web
