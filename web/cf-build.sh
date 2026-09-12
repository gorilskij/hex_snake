#!/usr/bin/env bash
# Cloudflare Pages build script.
#
# Set in the Pages project settings:
#   Build command:           bash web/cf-build.sh
#   Build output directory:  web
#   Production branch:        pub-website
#
# The Pages build image has no Rust toolchain, so install it, then build the
# wasm in release mode and drop it next to index.html. `hex_snake.wasm` is
# gitignored, so it is always produced fresh here rather than committed.
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi

# rust-toolchain.toml pins the nightly channel + wasm target; cargo installs
# both on this first invocation.
#
# The release profile keeps debuginfo (for native profiling), which balloons the
# wasm past Pages' 25 MiB per-file limit (~25 MiB vs ~1 MiB without), so drop
# it for the deployed build only.
CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/hex_snake.wasm web/hex_snake.wasm

echo "built web/hex_snake.wasm ($(du -h web/hex_snake.wasm | cut -f1))"
