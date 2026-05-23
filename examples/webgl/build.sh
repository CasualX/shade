#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

cargo build --release --package webgl --target wasm32-unknown-unknown
cp "$ROOT_DIR/target/wasm32-unknown-unknown/release/webgl.wasm" "$SCRIPT_DIR/html/webgl.wasm"

echo "Copied webgl.wasm to $SCRIPT_DIR/html/webgl.wasm"
