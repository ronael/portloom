#!/bin/sh
set -eu
ROOT=$(CDPATH= cd "$(dirname "$0")/../.." && pwd)
PREFIX=${1:-$HOME/.local}
if [ ! -x "$ROOT/bin/portloom" ]; then
  printf '%s\n' 'Compile d’abord avec npm run build.' >&2
  exit 1
fi
mkdir -p "$PREFIX/bin"
for NAME in portloom portscope; do
  DEST="$PREFIX/bin/$NAME"
  if [ -e "$DEST" ] || [ -L "$DEST" ]; then
    if [ ! -L "$DEST" ] || [ "$(readlink "$DEST")" != "$ROOT/bin/portloom" ]; then
      printf 'Commande déjà présente, conservée : %s\n' "$DEST" >&2
      exit 1
    fi
  fi
done
for NAME in portloom portscope; do
  if [ ! -L "$PREFIX/bin/$NAME" ]; then ln -s "$ROOT/bin/portloom" "$PREFIX/bin/$NAME"; fi
done
printf 'Commandes prêtes dans %s/bin : portloom, portscope\n' "$PREFIX"
