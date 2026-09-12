# hex_snake

A hex-grid snake game in Rust. Multiple snakes (player + AI), apples, a hexagonal
board, smooth animated rendering. Runs **natively and in the browser (wasm)** on a
single backend: **macroquad**.

> History: originally built on **ggez**. The `wasm` branch ported it to macroquad
> (native + web, no cfg split); the ggez-compat `gfx/` shim has since been
> dissolved into direct macroquad calls + `support/`. Snake coloring was moved to
> the GPU (see [Snake coloring](#snake-coloring-shader-based)), and the snake was
> reworked into a float-length model (see [Snake length model](#snake-length-model-snakemodrs)).
> Current work (`game-modes` branch) is game modes and new apple types.

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

`src/main.rs` — `#[macroquad::main]` async loop. Constructs a `Game` directly,
polls resize + `get_keys_pressed/released`, and drives the `Screen` trait
(`app/screen/mod.rs`): `update`/`draw`/`key_down_event`/`resize_event` with
macroquad's `KeyCode` passed straight through, then `next_frame().await`.

- Web delivers physical (qwerty-position) keycodes, so the player uses
  `Layout::Qwerty` (ul=J u=K ur=L dl=M d=Comma dr=Period).
- `register_custom_getrandom!` backs `rand`/`thread_rng` with macroquad's PRNG on
  wasm (native uses the OS backend).

## Module map

- **`support/`** — what used to be a `gfx/` ggez-compat layer, now dissolved: the
  game calls macroquad directly (`macroquad::color::Color`, `Material`,
  `clear_background`, cameras), and the few helpers that remain live here.
  - `mesh.rs` — **the core of rendering**: `Mesh`, `MeshBuilder` (lyon fill/
    stroke → chunked macroquad meshes), `DrawMode`, `build_circle`, and
    `set_board_camera` (the board-offset `Camera2D`). `mesh.draw()` /
    `mesh.draw_shaded(material)` replace the old `Canvas`.
  - `material.rs` — custom GLSL shader (`SnakeMaterial` via `snake_material()`) +
    palette LUT texture (`PaletteLut`) for snake coloring.
  - `time.rs` — `Instant` over `macroquad::time::get_time()` (std `Instant`
    panics on wasm). Also `partial_min_max` (NaN-safe), `flip`, `filter_scan`.
  - (The `Screen` trait — the old `EventHandler` — now lives in
    `app/screen/mod.rs`; `Game` implements it.)
- **`app/`** — game orchestration.
  - `screen/game.rs` — **the `Game` struct**: owns the world (`Environment`),
    `FpsControl`, cached meshes, and the `draw`/`update` event handlers. This is
    where rendering is assembled and drawn.
  - `screen/mod.rs` — `Environment<Rng>` (snakes, apples, portals, `GameContext`).
  - `fps_control.rs` — play/pause/game-over state, the world's per-frame
    time step (smoothed frame duration × global debug speed multiplier, stepped
    with `[` / `]`; `Game::update` splits fast frames into ticks of ≤ half a cell).
  - `game_context.rs` (`GameContext`: cell_dim, board_dim, prefs, palette),
    `prefs.rs` (`Prefs`; default draw_style = Smooth, grid+border on),
    `stats.rs`, `message.rs` (macroquad text overlay), `palette.rs` (board/bg
    colors, distinct from snake palette), `snake_management.rs` (advance, spawn,
    collisions, `outcome_at`), `border_hints.rs` (gradients on wrap-around edges
    hinting what the player would hit on the other side), `distance_grid.rs`,
    `portal/`, `board_dim.rs`.
  - `screen/{start_screen,snake_control_creator_screen,debug_scenario}.rs` — **out
    of the module tree** (not compiled); page chrome to be done in HTML/JS later.
- **`snake/`** — snake model. `mod.rs` (`Snake`, `Body`, `Segment`,
  `SegmentType`; the float-length model — see [Snake length model](#snake-length-model-snakemodrs)),
  `builder.rs`, `eat_mechanics.rs`, **`palette.rs`** (snake coloring: `Palette`
  trait, `SegmentStyle`, gradient/solid/alternating palettes, `build_snake_lut`).
- **`snake_control/`** — controllers: `keyboard`, AI `algorithm`/`killer`/`rain`/
  `programmed`, and `pathfinder/` (weighted BFS, space-filling, with-backup).
- **`rendering/`** — turns the world into meshes (see next section).
- **`basic/`** — `Point` (f32 x/y, cartesian), `HexPoint` (hex grid coord),
  `Dir`/`Dir12` (hex directions), `CellDim` (side/sin/cos, `height()`, `center()`),
  `board.rs`.
- **`color/`** — `Color` newtype over `macroquad::color::Color` with arithmetic
  ops; `oklab.rs`, `to_color.rs` (HSL→Color).
- **`view/`** — `OtherSnakes`/`Snakes` borrowing helpers (`split_snakes`).
- **`basic/`, `error.rs`, `keyboard_layout.rs`** — misc.
- **Out of tree (not compiled):** `button/`, `support/text_layout.rs`. Still on
  disk with stale `ggez` references; harmless.

## Snake length model (`snake/mod.rs`)

Conceptually a snake is a **float-length ribbon of material** flowing along its
trail at `speed` cells/s. `Body.segments` (a `VecDeque`) is just the stored
polyline for drawing/collision — the snake's length is **not** the segment
count. Head and tail are independent; nothing pins them to cell boundaries.

- **`length`** — *the conserved quantity*, the true length in cells (a float).
  Changed only by explicit, capped growth (digestion; `grow`/`shrink` later) —
  never as a drifting difference of accumulators. Birth and death do **not**
  touch it; they only change how much of it is on the board.
- **`head_fraction`** — the head's progress into its leading cell (0..1). A new
  head segment is pushed at each boundary crossing (in `advance_cell`).
- **`emerged` / `swallowed`** — the two **holes**, each measured in cells of
  material. *Birth:* material leaves the birth hole at head speed (`emerged`
  chases `length`), so the tail stays pinned at the hole until the snake is all
  the way out. *Death* (`state == Dying`, set by `die()`): the head pins at
  `HOLE_DEPTH` into its cell and the material flowing past drains into the death
  hole (`swallowed` grows). `on_board() = emerged − swallowed`. The holes are
  independent — a snake can emerge from one while vanishing into another.
- **The tail is derived, never stored:** `tail_fraction() = (visible_len − 1) +
  head_fraction − on_board()`; trailing segments pop once it passes 1. So the
  tail position cannot drift (only `length` is stored, and only additively).
- **Digestion:** an `Eaten { original_food, food_left }` tail segment is crossed
  at `1/(food+1)` speed, growing `length` by exactly `food` (capped by
  `food_left`, drift-free). Dying snakes keep digesting.

`advance(elapsed)` runs head-move → emerge → digest → pop-tail and returns
whether a cell boundary was crossed (→ caller invokes `advance_cell`). A dying
head never reaches a boundary, so `advance_cell` panics for `Dying`. A snake is
removed once `state == Dying && on_board() <= 0`.

> The old **black-hole graphic** (blue circle + a `SegmentType::BlackHole`
> marker) was **removed**; death now just recedes via this model with no visual.
> Proper birth/death **hole graphics** (hole opening/closing, snake fading to
> black) are to be reimplemented — keep the concept in mind.

## Rendering pipeline

`Game::draw` (`app/screen/game.rs`) lazily builds cached meshes and draws them in
z-order onto a `Canvas`:

1. Each `rendering::*_mesh` fn builds a `support::mesh::Mesh` (grid, border,
   apples, portals, distance grid, player path) via `MeshBuilder` (lyon fill/
   stroke → chunked macroquad meshes; chunks kept under macroquad's per-draw
   10000-vert / 5000-index clamp).
2. `set_board_camera` sets a board-offset `Camera2D` once (pixel coords,
   **y-down**; `zoom.y` must be positive), then `mesh.draw()` per mesh on the
   default material. (One camera set for all board meshes — each `set_camera`
   flushes the GPU batch, so per-mesh sets were the bulk of the frame's cost.)
3. Snakes are special — drawn through a shader (below).

Draw order (default material, split around the snake): distance_grid,
border_hints, grid, player_path → **snake (shaded)** → apple, border, portal →
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
  the existing `Palette`/`SegmentStyle` color functions into a LUT where each
  segment owns a fixed slot of `lut_texels_per_segment` texels (so segment
  boundaries sit on texel edges regardless of length). This reuses all the
  existing HSL/OkLab/gradient math; the LUT *is* the whole-body gradient.
- **Draw** (`rendering/snake_mesh.rs` → `SnakeRender`): one shaded `Mesh` + one
  `PaletteLut` per snake (LUT baked as the mesh's `texture`). `game.rs` compiles
  the `SnakeMaterial` (`snake_material()`) lazily and calls `mesh.draw_shaded` per
  snake. The fragment shader samples the LUT by `uv.x`, clamped to the segment's
  own slot (`normal.xy`), and scales it by a per-vertex brightness (`normal.z`;
  1 for the body, lower for details like passability marks) (linear filtered →
  smooth gradients; hard segment boundaries stay sharp). `uv.y` is currently
  unused (reserved for across-width shading).

Key files: `support/material.rs`, `snake/palette.rs` (`build_snake_lut`),
`rendering/snake_mesh.rs`, `rendering/segments/{smooth_segments/mod,point_factory,
hexagon_segments}.rs`, `rendering/segments/cap.rs` (round head/tail caps),
`rendering/segments/descriptions.rs` (`SegmentDescription`, `SegmentFraction`,
turn types).

### Round end caps (`rendering/segments/cap.rs`)

Smooth-style snakes get half-circle caps on both ends. `build_round_caps`
truncates the body ribbon by one cap radius (measured **along the body path**, so
caps straddle cell boundaries and bend around turns) and fills the gap with a
half-circle profile (`dist = radius·sin φ`, half-width `∝ cos φ`). A head
`Crashed` into an obstacle keeps its flat face; the radius shrinks for very short
snakes so the two caps don't overlap.

### Passability marks (`rendering/segments/marks.rs`)

Eaten segments a snake can pass through get marks in a **darker shade of the
segment's own color** (same LUT lookup, `BRIGHTNESS` = 0.8), so the player can
tell them from look-alikes they'd crash into (e.g. other snakes' eaten segments). Smooth style: two lines inset from the
edges with round ends, following the edges' curvature (arcs on turns, inner
shorter; a sharp turn's inner line is a dot). Where the neighboring segment is
marked too, the line runs flat to the shared edge, so consecutive marked
segments show one continuous line. The head segment joins ahead as soon as the
next segment is known to be eaten (`Snake::upcoming_eaten_segment`: direction
locked in + `Eat` apple in the next cell) — for now an instant jump, hidden
because lines are cut to the segment's drawn fraction (which stops where the
head cap begins); easing is still to be designed. Hexagon style: a small
hexagon in the middle. Dimensions are the `const`s at the top of the file (tuned
by eye).

Whether a segment is marked: `EatMechanics::is_marked(segment_type)` =
`mark_passable` flag (opt-in via `.mark_passable()`; only the player sets it) **and**
the snake's own behavior against that segment type is inert. So **marked ⇒
passable** by construction; not every passable segment is marked.

## Gotchas (each cost real time)

- **std `Instant::now()` panics on wasm** → `support::time::Instant` over macroquad's
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

- **Birth/death hole graphics** — the model exists (see [Snake length model](#snake-length-model-snakemodrs));
  the old black-hole circle was removed and nothing is drawn in its place yet.
  Reimplement as a hole opening/closing at the pinned end with the snake fading
  to black as it enters/leaves. Collision graphics (a crash effect) are a
  similar localized effect and want a shared approach.
- **Grow / shrink animation** — apples that change `length` should animate the
  change (eased) rather than snapping. Sketched as a single signed pending pool
  released into `length` over several frames; growth is capped at head speed (the
  tail can't reverse), shrink is capped by a max tail speed + a floor.
- **Perf:** LUT texture is recreated every frame per snake — switch to in-place
  `Texture2D::update` when size is unchanged. Consider a single draw call for all
  snakes via a LUT atlas + per-vertex snake index.
- **Cross-snake z-ordering:** shaded snakes are drawn per-snake in sequence, so
  the old global z-index interleaving across *different* snakes is not preserved
  (fine for the single-player game; revisit for multi-snake).
- **`uv.y` (across-width)** is emitted but unused — hook for tube/curvature
  shading later.
- **Deferred features from the wasm port:** FPS/stats overlay, on-canvas buttons,
  start screen, and the non-Game screens (to be HTML/JS page chrome).

## Deploy (separate, in progress)

Meant to be served at `games.gorilskij.com/hexsnake` behind a Cloudflare Worker
reverse-proxy (each game its own Pages project; the Astro site is the landing).
See `web/cf-build.sh` (Cloudflare Pages build: installs Rust, release-builds,
stages wasm) and the `pub-website`/`test-website` branches.
