#!/usr/bin/env bash
# epax install script — Linux & macOS
# Usage: curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/install.sh | bash

set -euo pipefail

REPO="podsni/epax"
BIN_NAME="epax"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ---------- detect platform ----------
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*)  OS_TAG="x86_64-unknown-linux-gnu" ;;
  Darwin*) ;;  # handled below
  *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

if [ "$OS" = "Darwin" ]; then
  case "$ARCH" in
    arm64|aarch64) OS_TAG="aarch64-apple-darwin" ;;
    x86_64)        OS_TAG="x86_64-apple-darwin" ;;
    *)             echo "Unsupported macOS arch: $ARCH"; exit 1 ;;
  esac
fi

# ---------- resolve version ----------
if [ -z "${VERSION:-}" ]; then
  echo "Fetching latest version..."
  VERSION="$(curl -sSf "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
fi

echo "Installing $BIN_NAME $VERSION for $OS_TAG"

# ---------- download & install ----------
ARCHIVE="epax-${OS_TAG}.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading $URL"
curl -sSfL "$URL" -o "$TMP/$ARCHIVE"
tar -xzf "$TMP/$ARCHIVE" -C "$TMP"

BINARY="$TMP/epax-${OS_TAG}/epax"
chmod +x "$BINARY"

if [ -w "$INSTALL_DIR" ]; then
  mv "$BINARY" "$INSTALL_DIR/$BIN_NAME"
else
  echo "Installing to $INSTALL_DIR (sudo required)"
  sudo mv "$BINARY" "$INSTALL_DIR/$BIN_NAME"
fi

echo ""
echo "$BIN_NAME $VERSION installed to $INSTALL_DIR/$BIN_NAME"
echo "Run: epax --help"
