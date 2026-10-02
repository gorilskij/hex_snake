# Native and web

One codebase, one backend: **macroquad** (miniquad underneath), natively and as wasm in a
browser. There is no `cfg` split in the game code except the few lines below.

## The loop

`src/main.rs`: a `#[macroquad::main]` async loop keeps the [screen stack](controls-and-screens.md#the-screens),
polls resizes and key presses (`KeyInput`, `get_keys_released`), drives the `Screen` trait
(`app/screen/mod.rs`: `update` / `draw` / `key_down_event` / `resize_event`), then
`next_frame().await`.

**Time** (`app/fps_control.rs`): play/pause/game-over state and the world's per-frame time step
(the smoothed frame duration × the global debug speed multiplier). `Game::update` splits fast
frames into ticks of at most half a cell. `support/time.rs` gives an `Instant` over macroquad's
clock (std's panics on wasm).

## Graphics

- **macOS defaults to OpenGL** (not Metal) and the web is WebGL, so one **GLSL ES 1.00** shader
  covers both (no MSL). Shaders must be WebGL1-clean (`attribute` / `varying` / `texture2D` /
  precision qualifiers).
- Shader compilation needs a GL window: a headless sandbox cannot validate shaders; use the
  browser.
- MSAA is on; the web canvas is Display P3 ([colour on the web](rendering.md#colour-on-the-web)).

## The wasm specifics

- **`.cargo/config.toml`** passes `--import-undefined` to the linker for the wasm target:
  macroquad/miniquad declare their JS-host functions (`init_webgl`, `gl*`, `console_log`…) as
  `extern "C"`, and recent rust-lld errors on them unless told to emit wasm imports, which the
  JS bundle satisfies at instantiation.
- **Randomness:** getrandom has no default backend on `wasm32-unknown-unknown`; the config
  selects the `custom` backend (`--cfg getrandom_backend="custom"`), implemented by
  `__getrandom_v03_custom` in `main.rs` over macroquad's PRNG (wasm only; natively the OS
  backend). The PRNG is seeded from the wall clock. **Not** `wasm_js` / `web-time` / `instant`:
  they pull in wasm-bindgen, which clashes with macroquad's JS loader. With `--import-undefined`
  a missing `__getrandom_v03_custom` would not fail the link — it would become a JS import and
  fail on page load — so check the wasm's imports after touching this.
- **Preferences** go to localStorage through `quad-storage-sys` (the `quad-storage.js` plugin and
  `sapp_jsutils.js`, which marshals strings across the wasm boundary).

## The page (`web/`)

| file | what |
|---|---|
| `index.html` | the page: a full-window `<canvas id="glcanvas">`, the Display P3 `getContext` wrapper, the scripts, `load("hex_snake.wasm")`. Inline JS is wrapped in an IIFE because the bundle declares a global `canvas` |
| `mq_js_bundle.js` | macroquad's JS loader, vendored and **patched** to map `MetaLeft` / `MetaRight` (upstream only knows the obsolete `OSLeft` / `OSRight`) |
| `sapp_jsutils.js`, `quad-storage.js` | the storage plugin (loaded after the bundle, which defines `miniquad_add_plugin`, and before `load()`) |
| `build.sh` | local builds: `cargo build --target wasm32-unknown-unknown [--release]`, copies the wasm to `web/hex_snake.wasm` (gitignored) |
| `cf-build.sh` | the Cloudflare build: installs Rust if missing, release build without debug info, stages `web/dist/games/hexsnake/` |
| `worker.js` | the Worker's script: misses go on to the site's 404 page ([deploy](../how-to/deploy.md)) |

A debug wasm is ~25 MB and far too slow to judge smoothness; release with debug info ~25 MB;
release without debug info (`CARGO_PROFILE_RELEASE_DEBUG=false`) ~2 MB.
