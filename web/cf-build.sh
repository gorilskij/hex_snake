#!/usr/bin/env bash
# Workers Builds build script (the Workers `hex-snake` and `hex-snake-test`).
#
# Set in each Worker's build settings:
#   Build command:   bash web/cf-build.sh
#   Deploy command:  npx wrangler deploy               (hex-snake, branch pub-website)
#                    npx wrangler deploy --env test    (hex-snake-test, branch test-website)
#
# The build image has no Rust toolchain, so install it, then build the wasm in
# release mode and stage the page into web/dist/games/hexsnake (the game is
# served at gorilskij.com/games/hexsnake/). `hex_snake.wasm` is gitignored, so
# it is always produced fresh here rather than committed.
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
# wasm past the 25 MiB per-file limit of static assets (~25 MiB vs ~1 MiB
# without), so drop it for the deployed build only.
CARGO_PROFILE_RELEASE_DEBUG=false cargo build --release --target wasm32-unknown-unknown

OUT=web/dist/games/hexsnake
rm -rf web/dist && mkdir -p "$OUT"
cp target/wasm32-unknown-unknown/release/hex-snake.wasm "$OUT/hex_snake.wasm"
cp web/index.html web/mq_js_bundle.js web/sapp_jsutils.js web/quad-storage.js "$OUT/"

echo "staged $OUT ($(du -sh web/dist | cut -f1))"
