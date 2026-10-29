#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/generate-rpc-clients.sh --check
pnpm --dir typescript --filter @chapchoo/rpc-client test
python3 scripts/run-rust-tests.py --rpc-clients
