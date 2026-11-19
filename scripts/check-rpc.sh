#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/rust-check-env.sh
bash scripts/generate-rpc-clients.sh --check
pnpm --filter @chapchoo/rpc-client build
python3 scripts/run-rust-tests.py --rpc-clients
