#!/usr/bin/env bash
# epax uninstall script — Linux & macOS
# Usage: curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/uninstall.sh | bash

set -euo pipefail

BIN_NAME="epax"
PURGE=false
FORCE=false

for arg in "$@"; do
  case "$arg" in
    --purge) PURGE=true ;;
    --force) FORCE=true ;;
  esac
done

# find the binary
BIN_PATH="$(command -v "$BIN_NAME" 2>/dev/null || true)"
if [ -z "$BIN_PATH" ]; then
  # fallback common locations
  for d in /usr/local/bin /usr/bin "$HOME/.local/bin"; do
    if [ -x "$d/$BIN_NAME" ]; then BIN_PATH="$d/$BIN_NAME"; break; fi
  done
fi

if [ -z "$BIN_PATH" ]; then
  echo "$BIN_NAME is not installed (not found in PATH or common locations)."
  exit 0
fi

if [ "$FORCE" = false ]; then
  printf "Uninstall %s at %s? [y/N] " "$BIN_NAME" "$BIN_PATH"
  read -r answer
  case "$answer" in [yY]*) ;; *) echo "Aborted."; exit 0 ;; esac
fi

if [ -w "$BIN_PATH" ]; then
  rm "$BIN_PATH"
else
  sudo rm "$BIN_PATH"
fi
echo "Removed $BIN_PATH"

if [ "$PURGE" = true ]; then
  for cfg in "$HOME/.config/epax" "$HOME/.local/share/epax"; do
    if [ -d "$cfg" ]; then
      rm -rf "$cfg"
      echo "Removed $cfg"
    fi
  done
fi

echo "$BIN_NAME uninstalled."
