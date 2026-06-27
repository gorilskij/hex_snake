#!/usr/bin/env bash
# Build the wasm binary and stage it next to index.html, then (optionally) serve.
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="${1:-debug}"
if [ "$PROFILE" = "release" ]; then
    cargo build --target wasm32-unknown-unknown --release
    cp target/wasm32-unknown-unknown/release/hex_snake.wasm web/hex_snake.wasm
else
    cargo build --target wasm32-unknown-unknown
    cp target/wasm32-unknown-unknown/debug/hex_snake.wasm web/hex_snake.wasm
fi

echo "staged web/hex_snake.wasm ($(du -h web/hex_snake.wasm | cut -f1))"
echo "serve with:  (cd web && python3 -m http.server 4000)"
