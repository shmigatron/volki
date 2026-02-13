#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

INSTALL_DIR="${1:-/usr/local/bin}"
BIN="$ROOT_DIR/target/release/volki"

if [ ! -f "$BIN" ]; then
    echo "Release binary not found. Building first..."
    "$SCRIPT_DIR/build.sh" release
fi

echo "Installing volki to $INSTALL_DIR..."
install -m 755 "$BIN" "$INSTALL_DIR/volki"
echo "Installed: $(which volki || echo "$INSTALL_DIR/volki")"
echo "Version: $(volki --version 2>/dev/null || echo 'installed')"
