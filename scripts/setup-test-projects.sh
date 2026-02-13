#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOCAL_DIR="$ROOT_DIR/tests/local"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

pass=0
fail=0
skip=0

setup_ecosystem() {
    local name="$1"
    local dir="$2"
    local cmd_check="$3"
    local setup_cmd="$4"

    printf "${CYAN}[%s]${NC} " "$name"

    if ! command -v "$cmd_check" &>/dev/null; then
        printf "${YELLOW}skipped${NC} (%s not found)\n" "$cmd_check"
        ((skip++))
        return
    fi

    if (cd "$dir" && eval "$setup_cmd") &>/dev/null; then
        printf "${GREEN}ok${NC}\n"
        ((pass++))
    else
        printf "${RED}failed${NC}\n"
        ((fail++))
    fi
}

echo "Setting up test projects in $LOCAL_DIR"
echo "========================================"
echo ""

# Node.js
setup_ecosystem "node" "$LOCAL_DIR/node" "npm" "npm install --ignore-scripts"

# Python
printf "${CYAN}[python]${NC} "
if command -v python3 &>/dev/null; then
    if (cd "$LOCAL_DIR/python" && python3 -m venv .venv && .venv/bin/pip install -q requests click pydantic) &>/dev/null; then
        printf "${GREEN}ok${NC}\n"
        ((pass++))
    else
        printf "${RED}failed${NC}\n"
        ((fail++))
    fi
else
    printf "${YELLOW}skipped${NC} (python3 not found)\n"
    ((skip++))
fi

# Ruby
setup_ecosystem "ruby" "$LOCAL_DIR/ruby" "bundle" "bundle install --path vendor/bundle"

# Rust
setup_ecosystem "rust" "$LOCAL_DIR/rust" "cargo" "cargo generate-lockfile"

# Go
setup_ecosystem "go" "$LOCAL_DIR/go" "go" "go mod download"

# Java (Maven)
setup_ecosystem "java-maven" "$LOCAL_DIR/java-maven" "mvn" "mvn dependency:resolve -q"

# Java (Gradle)
setup_ecosystem "java-gradle" "$LOCAL_DIR/java-gradle" "gradle" "gradle dependencies --no-daemon -q"

# .NET
setup_ecosystem "dotnet" "$LOCAL_DIR/dotnet" "dotnet" "dotnet restore"

# PHP
setup_ecosystem "php" "$LOCAL_DIR/php" "composer" "composer install --no-interaction --quiet"

# Elixir
setup_ecosystem "elixir" "$LOCAL_DIR/elixir" "mix" "mix deps.get"

# Swift
setup_ecosystem "swift" "$LOCAL_DIR/swift" "swift" "swift package resolve"

# Dart
setup_ecosystem "dart" "$LOCAL_DIR/dart" "dart" "dart pub get"

echo ""
echo "========================================"
printf "Done: ${GREEN}%d passed${NC}" "$pass"
if [ "$skip" -gt 0 ]; then printf ", ${YELLOW}%d skipped${NC}" "$skip"; fi
if [ "$fail" -gt 0 ]; then printf ", ${RED}%d failed${NC}" "$fail"; fi
echo ""
