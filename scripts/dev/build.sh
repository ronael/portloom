#!/bin/sh
set -eu
ROOT=$(CDPATH= cd "$(dirname "$0")/../.." && pwd)
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) HOST=aarch64-apple-darwin;; Darwin-x86_64) HOST=x86_64-apple-darwin;;
  Linux-aarch64) HOST=aarch64-unknown-linux-gnu;; Linux-x86_64) HOST=x86_64-unknown-linux-gnu;;
  *) printf '%s\n' 'Plateforme non prise en charge.' >&2; exit 1;;
esac
TARGET=${1:-$HOST}
case "$TARGET" in
  aarch64-apple-darwin) PLATFORM=darwin-arm64;; x86_64-apple-darwin) PLATFORM=darwin-x64;;
  aarch64-unknown-linux-gnu) PLATFORM=linux-arm64;; x86_64-unknown-linux-gnu) PLATFORM=linux-x64;;
  *) printf '%s\n' 'Cible inconnue.' >&2; exit 1;;
esac
sh "$ROOT/scripts/dev/rust.sh" build --release --locked --bin portloom --target "$TARGET"
SOURCE="${CARGO_TARGET_DIR:-$ROOT/.tooling/target}/$TARGET/release/portloom"
mkdir -p "$ROOT/dist/native/$PLATFORM"
cp "$SOURCE" "$ROOT/dist/native/$PLATFORM/portloom"
chmod 755 "$ROOT/dist/native/$PLATFORM/portloom"
mkdir -p "$ROOT/.tooling"
sh "$ROOT/scripts/dev/rust.sh" metadata --locked --format-version 1 --filter-platform "$TARGET" > "$ROOT/.tooling/metadata-$PLATFORM.json"
node "$ROOT/scripts/release/notices.mjs" "$PLATFORM" "$ROOT/.tooling/metadata-$PLATFORM.json"
node "$ROOT/scripts/release/record.mjs" "$PLATFORM" "$TARGET"
if [ "$TARGET" = "$HOST" ]; then
  mkdir -p "$ROOT/bin"
  cp "$SOURCE" "$ROOT/bin/.portloom-next"
  chmod 755 "$ROOT/bin/.portloom-next"
  mv -f "$ROOT/bin/.portloom-next" "$ROOT/bin/portloom"
  ln -sfn portloom "$ROOT/bin/portscope"
fi
printf 'Binaire prêt : %s\n' "$ROOT/dist/native/$PLATFORM/portloom"
