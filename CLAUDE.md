# hex_snake

A hex-grid snake game in Rust. Multiple snakes (player + AI), apples, a hexagonal
board, smooth animated rendering. Runs **natively and in the browser (wasm)** on a
single backend: **macroquad**.

> History: originally built on **ggez**. The `wasm` branch ported it to macroquad
> (native + web, no cfg split). The current `shaders` branch reworks snake
> coloring to run on the GPU (see [Snake coloring](#snake-coloring-shader-based)).

## Build / run / test

- **Native:** `cargo run` (fast iteration; but shader compilation needs a GL
  window, so a headless sandbox can't validate shaders — use the browser for
  that).
- **Wasm:** `./web/build.sh` (debug) or `./web/build.sh release`. Builds
  `wasm32-unknown-unknown` and copies the artifact to `web/hex_snake.wasm`
  (gitignored, ~25 MB debug).
- **Serve:** `(cd web && python3 -m http.server 4000)` → open
  `http://127.0.0.1:4000/index.html`. `web/` holds `index.html` + vendored
  `mq_js_bundle.js` (canvas id `glcanvas`; inline JS is wrapped in an IIFE because
  the bundle declares a global `canvas`).
- **Toolchain:** nightly (see `rust-toolchain.toml`; the crate uses several
  `#![feature(...)]`). `.cargo/config.toml` passes `--import-undefined` for the
  wasm target (miniquad's JS-host `extern "C"` symbols).

Requires nightly for: `stmt_expr_attributes`, `try_blocks`,
`exhaustive_patterns`, `if_let_guard` (see `src/main.rs`).

## Entry point & main loop

`src/main.rs` — `#[macroquad::main]` async loop. Constructs a `Game` directly
(bypassing the old `App`/`Screen` machinery), polls resize + `get_keys_pressed/
released`, maps macroquad keycodes → `gfx::input::keyboard::KeyCode`, calls
`update`/`draw`, `next_frame().await`.

- Web delivers physical (qwerty-position) keycodes, so the player uses
  `Layout::Qwerty` (ul=J u=K ur=L dl=M d=Comma dr=Period).
- `register_custom_getrandom!` backs `rand`/`thread_rng` with macroquad's PRNG on
  wasm (native uses the OS backend).
- Contains a `SPIKE` flag + spike draw calls — **temporary** shader-validation
  scaffolding, remove during cleanup (see TODOs).

## Module map

- **`gfx/`** — thin compatibility layer mimicking the small `ggez` API surface the
  game uses, backed by macroquad. Porting a file was mostly repointing
  `use ggez::…` → `use crate::gfx::…`.
  - `graphics.rs` — `Color`, `MeshBuilder`, `Mesh`, `Canvas`, `DrawParam`,
    lyon-based tessellation, the board `Camera2D`. **The core of the gfx layer.**
  - `material.rs` — custom GLSL shader (`SnakeMaterial`) + palette LUT texture
    (`PaletteLut`) for snake coloring. (shaders branch)
  - `input.rs` (keyboard/mouse), `event.rs` (`EventHandler` trait), `time.rs`
    (`Instant` over `macroquad::time::get_time()` — std `Instant` panics on wasm),
    `mod.rs` (`Context`, `GameError`/`GameResult`).
- **`app/`** — game orchestration.
  - `screen/game.rs` — **the `Game` struct**: owns the world (`Environment`),
    `FpsControl`, cached meshes, and the `draw`/`update` event handlers. This is
    where rendering is assembled and drawn.
  - `screen/mod.rs` — `Environment<Rng>` (snakes, apples, portals, `GameContext`).
  - `fps_control.rs` — game-frame accumulation, `frame_fraction`, `can_update`.
  - `game_context.rs` (`GameContext`: cell_dim, board_dim, prefs, palette),
    `prefs.rs` (`Prefs`; default draw_style = Smooth, grid+border on),
    `stats.rs`, `message.rs` (macroquad text overlay), `palette.rs` (board/bg
    colors, distinct from snake palette), `snake_management.rs` (advance, spawn,
    collisions), `distance_grid.rs`, `portal/`, `board_dim.rs`.
  - `screen/{start_screen,snake_control_creator_screen,debug_scenario}.rs` — **out
    of the module tree** (not compiled); page chrome to be done in HTML/JS later.
- **`snake/`** — snake model. `mod.rs` (`Snake`, `Body`, `Segment`,
  `SegmentType`), `builder.rs`, `eat_mechanics.rs`, **`palette.rs`** (snake
  coloring: `Palette` trait, `SegmentStyle`, gradient/solid/alternating palettes,
  `build_snake_lut`).
- **`snake_control/`** — controllers: `keyboard`, AI `algorithm`/`killer`/`rain`/
  `programmed`, and `pathfinder/` (weighted BFS, space-filling, with-backup).
- **`rendering/`** — turns the world into meshes (see next section).
- **`basic/`** — `Point` (f32 x/y, cartesian), `HexPoint` (hex grid coord),
  `Dir`/`Dir12` (hex directions), `CellDim` (side/sin/cos, `height()`, `center()`),
  `board.rs`.
- **`color/`** — `Color` newtype over `gfx::graphics::Color` with arithmetic ops;
  `oklab.rs`, `to_color.rs` (HSL→Color).
- **`view/`** — `OtherSnakes`/`Snakes` borrowing helpers (`split_snakes`).
- **`support/`** — `partial_min_max` (NaN-safe), `flip`, `invert`, etc.
- **`basic/`, `error.rs`, `keyboard_layout.rs`** — misc.
- **Out of tree (not compiled):** `button/`, `support/text_layout.rs`. Still on
  disk with stale `ggez` references; harmless.

## Rendering pipeline

`Game::draw` (`app/screen/game.rs`) lazily builds cached meshes and draws them in
z-order onto a `Canvas`:

1. Each `rendering::*_mesh` fn builds a `gfx::graphics::Mesh` (grid, border,
   apples, portals, distance grid, player path) via `MeshBuilder` (lyon fill/
   stroke → chunked macroquad meshes; chunks kept under macroquad's per-draw
   10000-vert / 5000-index clamp).
2. `Canvas::draw` sets a board-offset `Camera2D` (pixel coords, **y-down**;
   `zoom.y` must be positive) then `draw_mesh` per chunk on the default material.
3. Snakes are special — drawn through a shader (below).

Draw order (default material, split around the snake): distance_grid, grid,
player_path → **snake (shaded)** + black-hole circles → apple, border, portal →
message text.

### Snake coloring (shader-based)

The snake's hue gradient used to be faked by chopping each segment into ~20 flat-
colored "subsegments" (CPU, re-running arc geometry per slice — the perf hog).
That's **gone**. Now:

- **Geometry** (`rendering/segments/`): each segment is **one polygon**, generated
  as a **triangle-strip ribbon of cross-sections** (`smooth_segments::
  segment_cross_sections`). A straight segment = 2 cross-sections; a turn = an arc
  of them. Each cross-section is `(inner, outer, frac)` in *default orientation*.
  `point_factory::build_shaded` normalizes `frac` → global body coordinate
  `uv.x = (seg_idx + (1 - frac)) / num_segments`, applies the flip/rotate/
  translate to positions, and emits vertices via `MeshBuilder::push_shaded_ribbon`
  (macroquad `Vertex` carries `uv.x` = along-body, `uv.y` = across-width).
  - Adjacent cross-sections **share** vertices, so radial edges are single
    constant-`frac` lines → **seamless**. A complete sharp turn has
    `inner_radius == 0`: the inner point is just the pivot, reused across
    cross-sections with different `frac` — a true single point, no hole, no seam.
  - Hexagon draw style (`hexagon_segments::hexagon_outline`) is a flat hexagon
    with constant `uv.x` (via `push_shaded_polygon`).
- **Color** (`snake/palette.rs`): per snake per frame, `build_snake_lut` samples
  the existing `Palette`/`SegmentStyle` color function into a fixed **2048-texel
  1-D LUT** (`SNAKE_LUT_SIZE`). This reuses all the existing HSL/OkLab/gradient
  math; the LUT *is* the whole-body gradient.
- **Draw** (`rendering/snake_mesh.rs` → `SnakeRender`): one shaded `Mesh` + one
  `PaletteLut` per snake (LUT baked as the mesh's `texture`). `game.rs` compiles
  the `SnakeMaterial` lazily and calls `Canvas::draw_shaded` per snake. The
  fragment shader samples the LUT by `uv.x` (linear filtered → smooth gradients;
  dense enough that hard segment boundaries stay ~sharp). `uv.y` is currently
  unused (reserved for across-width shading).

Key files: `gfx/material.rs`, `snake/palette.rs` (`build_snake_lut`),
`rendering/snake_mesh.rs`, `rendering/segments/{smooth_segments/mod,point_factory,
hexagon_segments}.rs`, `rendering/segments/descriptions.rs` (`SegmentDescription`,
`SegmentFraction`, turn types).

## Gotchas (each cost real time)

- **std `Instant::now()` panics on wasm** → `gfx::time::Instant` over macroquad's
  clock; `fps_control` uses it.
- **macroquad clamps each `draw_mesh`** to 10000 verts / 5000 indices, silently
  dropping overflow. `Mesh::from_data` chunks under that.
- **Board camera is y-down** for `draw_mesh`; `zoom.y` positive (negative flips the
  board — the snake appears to move the wrong way).
- **lyon triangulates arbitrarily** — do NOT feed a curved outline to lyon fill and
  derive per-vertex attributes from position; triangles cross constant-color lines
  → seams. Use the explicit cross-section ribbon instead.
- **`atan2` wraps at ±π** — the old position-based uv approach hit this at the turn
  start seam (now avoided by generating `frac` directly per cross-section).
- **macOS defaults to OpenGL** (not Metal), and web is WebGL → a single **GLSL-ES
  100** shader covers both (no MSL needed). Shaders must be WebGL1-clean
  (`attribute`/`varying`/`texture2D`/precision qualifiers).
- **`Texture2D` is Arc-managed** (frees GPU on last drop), so rebuilding the LUT
  each frame doesn't leak — but it does churn a GPU texture per snake per frame
  (see TODO).
- **getrandom on wasm:** use the `custom` feature only; NOT `js`/`web-time`/
  `instant` (they pull wasm-bindgen, which clashes with macroquad's JS loader).

## TODOs / not yet done

- **Round head** — the snake front is currently flat (the old rounded cap was
  entangled with subsegmentation and removed). To be reimplemented as proper
  single-polygon geometry.
- **Cleanup:** remove the `SPIKE` scaffolding in `main.rs` and the spike helpers
  in `gfx/material.rs`; drop `push_shaded_polygon` if hexagon style is retired;
  remove now-stale `Polygon`/`RoundHeadDescription` in
  `rendering/segments/descriptions.rs`.
- **Perf:** LUT texture is recreated every frame per snake — switch to in-place
  `Texture2D::update` when size is unchanged. Consider a single draw call for all
  snakes via a LUT atlas + per-vertex snake index.
- **Cross-snake z-ordering:** shaded snakes are drawn per-snake in sequence, so the
  old global z-index/black-hole interleaving across *different* snakes is not
  preserved (fine for the single-player game; revisit for multi-snake).
- **`uv.y` (across-width)** is emitted but unused — hook for tube/curvature
  shading later.
- **Deferred features from the wasm port:** FPS/stats overlay, on-canvas buttons,
  start screen, and the non-Game screens (to be HTML/JS page chrome).

## Deploy (separate, in progress)

Meant to be served at `games.gorilskij.com/hexsnake` behind a Cloudflare Worker
reverse-proxy (each game its own Pages project; the Astro site is the landing).
See `web/cf-build.sh` (Cloudflare Pages build: installs Rust, release-builds,
stages wasm) and the `pub-website`/`test-website` branches.
