#!/usr/bin/env bash
set -euo pipefail

echo "=== Building DeepSeek Harness Desktop Packages ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$ROOT_DIR"

# 1. Bundle runtime assets
bash scripts/bundle_runtime.sh

# 2. Build release binary
echo "Compiling release binary..."
cargo build --release -p dsh-ui

echo "✓ Release binary built at target/release/dsh-desktop"
