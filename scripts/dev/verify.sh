#!/bin/sh
set -eu
ROOT=$(CDPATH= cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"
node scripts/dev/architecture.mjs
sh scripts/dev/rust.sh fmt --all -- --check
sh scripts/dev/rust.sh clippy --locked --workspace --all-targets -- -D warnings
sh scripts/dev/rust.sh test --locked --workspace --all-targets
node --test tests/distribution/*.test.mjs
