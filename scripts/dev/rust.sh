#!/bin/sh
set -eu
ROOT=$(CDPATH= cd "$(dirname "$0")/../.." && pwd)
if [ -x "$ROOT/.tooling/cargo/bin/cargo" ]; then
  TOOLING=$(CDPATH= cd "$ROOT/.tooling" && pwd -P)
  export CARGO_HOME="$TOOLING/cargo"
  export RUSTUP_HOME="$TOOLING/rustup"
  CARGO="$CARGO_HOME/bin/cargo"
else
  CARGO=cargo
fi
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/.tooling/target}"
cd "$ROOT"
exec "$CARGO" "$@"
