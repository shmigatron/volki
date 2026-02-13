#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

MODE="${1:-release}"

case "$MODE" in
    release)
        echo "Building volki (release)..."
        cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"
        BIN="$ROOT_DIR/target/release/volki"
        ;;
    debug)
        echo "Building volki (debug)..."
        cargo build --manifest-path "$ROOT_DIR/Cargo.toml"
        BIN="$ROOT_DIR/target/debug/volki"
        ;;
    *)
        echo "Usage: $0 [release|debug]"
        exit 1
        ;;
esac

SIZE=$(du -h "$BIN" | cut -f1)
echo "Built: $BIN ($SIZE)"
