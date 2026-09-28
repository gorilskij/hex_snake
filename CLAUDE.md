# hex_snake

A hex-grid snake game in Rust. Multiple snakes (player + AI), apples, a hexagonal
board, smooth animated rendering. Runs **natively and in the browser (wasm)** on a
single backend: **macroquad**.

> History: originally built on **ggez**. The `wasm` branch ported it to macroquad
> (native + web, no cfg split); the ggez-compat `gfx/` shim has since been
> dissolved into direct macroquad calls + `support/`. Snake coloring was moved to
> the GPU (see [Snake coloring](#snake-coloring-shader-based)), and the snake was
> reworked into a float-length model (see [Snake length model](#snake-length-model-snakemodrs)).
> Game modes and new apple types came next: Hunger mode (`app/game_mode.rs`)
> is implemented and picked on the start screen. Most
> recently (`menus` branch, merged): a start screen, in-game menus and
> rebindable, layout-aware controls — see [Screens & menus](#screens--menus).
> In progress (`graphics-updates` branch): lighting the snake as a
> round tube with pill-shaped ends and the apples as balls, and an OkLab
> constant-lightness rainbow — see [Snake coloring](#snake-coloring-shader-based).

## Build / run / test

- **Native:** `cargo run` (fast iteration; but shader compilation needs a GL
  window, so a headless sandbox can't validate shaders — use the browser for
  that).
- **Wasm:** `./web/build.sh` (debug) or `./web/build.sh release`. Builds
  `wasm32-unknown-unknown` and copies the artifact to `web/hex_snake.wasm`
  (gitignored, ~25 MB debug).
- **Release for the web:** `./web/build.sh release` — the default debug build
  is far too slow in wasm to judge smoothness. Add
  `CARGO_PROFILE_RELEASE_DEBUG=false` for a ~2 MB file instead of ~25 MB.
- **Serve:** `(cd web && python3 -m http.server 4000)` → open
  `http://127.0.0.1:4000/index.html`. `web/` holds `index.html` + vendored
  `mq_js_bundle.js` (canvas id `glcanvas`; inline JS is wrapped in an IIFE because
  the bundle declares a global `canvas`).
- **Toolchain:** nightly (see `rust-toolchain.toml`; the crate uses several
  `#![feature(...)]`). `.cargo/config.toml` passes `--import-undefined` for the
  wasm target (miniquad's JS-host `extern "C"` symbols).

- **Benchmark:** `cargo test --release benchmark -- --ignored --nocapture`
  (`app/benchmark.rs`): the headless world update with 24 apple-seeking
  snakes on an 80x50 board with 80 apples, timing planning apart from the
  rest.

Requires nightly for: `stmt_expr_attributes`, `try_blocks`,
`exhaustive_patterns`, `if_let_guard` (see `src/main.rs`).

## Entry point & main loop

`src/main.rs` — `#[macroquad::main]` async loop. Keeps a stack of screens
(`StartScreen` at the bottom; starting a game pushes a `Game`, which pops itself
to return to the main menu — see `Transition`),
polls resize + key presses (`app/key.rs`'s `KeyInput`) / `get_keys_released`, and drives the `Screen` trait
(`app/screen/mod.rs`): `update`/`draw`/`key_down_event`/`resize_event` with
each layout-aware `Key` (every key check, debug keys included, goes by
what the layout types — see below), then `next_frame().await`.

- **Keys follow the layout** (`app/key.rs`): key codes are positional on macOS,
  Windows and the web (Linux X11 is layout-based), so a key that types
  something is identified by the character it types (`Key::Char`, uppercase),
  paired from miniquad's ordered key-down + char events; other keys keep their
  code (`Key::Code`). Players' bindings live in `Prefs` (left/right player,
  plus which one a single player uses), edited from the options menu's
  Controls screen (both players side by side); the keyboard controller looks
  them up on every press.
- All text uses the bundled DejaVu Sans (`support/text.rs`,
  `assets/fonts/`); macroquad's default font is ASCII-only.
- `web/mq_js_bundle.js` is patched to map `MetaLeft`/`MetaRight` (the upstream
  bundle only knows the obsolete `OSLeft`/`OSRight`).
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
    panics on wasm). Also `partial_min_max` (NaN-safe), `flip`.
  - `text.rs` — all text, in the bundled font, rasterized at a few fixed sizes
    and scaled (see Gotchas). `storage.rs` — the saved-preferences blob
    (localStorage on the web, a config file natively).
  - (The `Screen` trait — the old `EventHandler` — now lives in
    `app/screen/mod.rs`; `Game` implements it.)
- **`app/`** — game orchestration.
  - `screen/game.rs` — **the `Game` struct**: owns the world (`Environment`),
    `FpsControl`, cached meshes, the menu, and the `draw`/`update` event
    handlers. This is where rendering is assembled and drawn.
  - `screen/mod.rs` — the `Screen` trait, `Transition` (the screen stack), and
    `Environment<Rng>` (snakes, apples, portals, `GameContext`).
  - `screen/start_screen.rs`, `screen/{menu,options_menu,controls_menu}.rs` —
    see [Screens & menus](#screens--menus). `key.rs` — layout-aware keys and
    player bindings.
  - `fps_control.rs` — play/pause/game-over state, the world's per-frame
    time step (smoothed frame duration × global debug speed multiplier, stepped
    with `[` / `]`; `Game::update` splits fast frames into ticks of ≤ half a cell).
  - `game_context.rs` (`GameContext`: cell_dim, board_dim, prefs, palette, mode),
    `game_mode.rs` (`GameMode` — Classic / Hunger — and the `hunger` tuning
    constants; picked on the start screen, which sets each player seed's
    starvation from it when the game starts),
    `prefs.rs` (`Prefs`, saved on every change: display settings, key
    bindings, single-player side; default draw_style = Smooth, grid+border on),
    `stats.rs` (the stats overlay: polygons drawn this frame, counted by
    each `Mesh`, + the player's exact `length`), `message.rs` (text overlays, multi-line, optionally on a
    dimmed box), `palette.rs` (board/bg
    colors, distinct from snake palette), `snake_management.rs` (advance, spawn,
    collisions — see [Collision](#collision-appsnake_managementrs) — and
    `outcome_at`), `border_hints.rs` (marks wrap-around edges —
    by what the player would hit on the other side, as recolored border
    stretches or gradients into the cell; or by where the head would come out,
    recoloring both ends of each wrap it could reach (one per direction) and
    growing a triangle inwards from the arrival end as it closes, both ends
    sweeping down the hues from purple to red in step with the triangle; or
    by the way out, as one translucent grey line per axis from the head out to
    the borders, drawn over the snake — snapped to the head's cell, or through
    the tip of the drawn head (`Centerline::head_tip`, the cap's apex, so the
    origin follows the arc through a turn), in which case each end is clipped
    against the board's zigzag edge and slides along it; or as a light in the
    dark — `HintStyle::Light` leaves the grid and border unseen except where
    lights on the border show them (`border_lights` → `light_material`,
    through which `game.rs` draws the grid and border): one per side of the
    board, level with the head's tip on a straight line through the middle of
    that side's zigzag (the nearest point on the zigzag itself hops from tooth
    to tooth), brighter as the head closes in from `LIGHT_RANGE` (3 cells),
    reaching `LIGHT_RADIUS` (3 cells); and one per direction where the head
    would come out, as bright as where it would leave: every line of cells
    wraps onto itself, so that is where the line through the head's tip
    leaves the smoothed border going the other way (all six directions, so
    none pops on or off as the head turns). `HintStyle`, cycled from the
    options menu), `distance_grid.rs`,
    `portal/`, `board_dim.rs`.
  - `screen/snake_control_creator_screen.rs` — **out of the module tree**
    (not compiled).
- **`snake/`** — snake model. `mod.rs` (`Snake`, `Body`, `Segment`,
  `SegmentType`; the float-length model — see [Snake length model](#snake-length-model-snakemodrs)),
  `builder.rs`, `eat_mechanics.rs`, **`palette.rs`** (snake coloring: `Palette`
  trait, `SegmentStyle`, gradient/solid/alternating palettes, `build_snake_lut`).
- **`snake_control/`** — controllers: `keyboard`, AI `algorithm`/`killer`/`rain`/
  `programmed`, and `pathfinder/`: cheapest-path search (`WeightedBFS`, costs
  in a `Weights` struct — step, blunt/sharp turn, teleport, pass-through), a
  space-filling survival fallback, with-backup, and `Obstacles` (occupied cells
  judged by the snake's `eat_self` for its own segments, `eat_other` for other
  snakes'). A search finds **one `Leg`** — the cheapest way from a start state
  `(cell, heading)` to any target not already taken, around what the plan has
  already `Committed` to — and chaining legs into a `Plan` is **path
  management**, done by `apple_seeker.rs`, not by the searches: the head walks
  the plan off the front and a new leg is appended at the back as targets are
  eaten, so the committed part of the route never moves. `Leg::target` is
  `None` for the survival crawl, which is what makes a plan a fallback (and is
  only ever accepted as a plan's only leg). The crawl is up to 20 cells, each
  step into the free neighbour with the most room behind it; it is walked, not
  recomputed, until an `Opening` says a target may be reachable: a blocked cell
  walling in the reachable area frees up, or a new apple appears inside it. The route is an obstacle to its own
  later legs — cells it uses are `Blocked`, except targets, which will hold an
  eaten segment and are `Passable` when the snake's own eat mechanics say it
  can pass through one. An apple that appears **on** the route is promoted to a
  target by splitting that leg in two at its cell, leaving the route itself
  untouched; that can push a plan past its target count, and extending waits
  until it is back under. The player's autopilot plans 3 targets ahead
  (`autopilot_targets`, `main.rs`); other snakes plan 1. What a search goes
  for comes from the autopilot's `Appetite` (`appetite.rs`, per apple kind, as
  `Knowledge` is per segment kind): positive apples are `Goals::targets`,
  negative ones `Goals::avoid` — that much extra cost to cross their cell. The
  player's autopilot avoids shrink apples at −15 (`autopilot_appetite`); one
  landing on the route triggers a single replan. Border hints show them as
  `Outcome::BadApple`, in the bad apples' green.
- **`rendering/`** — turns the world into meshes (see next section).
- **`basic/`** — `Point` (f32 x/y, cartesian), `HexPoint` (hex grid coord),
  `Dir`/`Dir12` (hex directions), `CellDim` (side/sin/cos, `height()`, `center()`),
  `board.rs`.
- **`color/`** — `Color` newtype over `macroquad::color::Color` with arithmetic
  ops; `oklab.rs` (OkLab ↔ sRGB, with the sRGB transfer function, and
  `max_chroma`: the most chroma a lightness + hue has in gamut, by bisection),
  `to_color.rs` (HSL→Color).
- **`view/`** — `OtherSnakes`/`Snakes` borrowing helpers (`split_snakes`).
- **`basic/`, `error.rs`** — misc.
- **`button/`** — immediate-mode polygon buttons (`Button`, `ButtonData`:
  outline, inner shapes, text spans; hit-tested on the exact outline) and the
  shared `style` (colors: grey, green hover, `ACCENT` yellow when pressed).
- **Out of tree (not compiled):** `support/text_layout.rs`. Still on disk with
  stale `ggez` references; harmless.

## Screens & menus

- **Screen stack** (`main.rs`, `Transition`): `StartScreen` stays at the bottom;
  starting a game pushes a `Game`; "Main menu" pops it, back to the start screen
  as it was left (`resume` reloads prefs, which the game may have changed).
- **Start screen:** players (one/two), mode (Classic/Hunger, not saved) and
  Options buttons, one demo per player
  (a hexagonal board one cell wider than the snake's programmed loop, defined
  by a center + loop radius; follows the snake style, grid and border
  settings; grid/border built once per size), palette arrows under each. ←/→
  change palettes only with one player; Enter starts; Esc opens the options.
- **Menus** (`menu.rs`, over a 90% black layer; the game is paused while open
  and stays paused after): options (`options_menu.rs`: lines of hexagon
  buttons, scrolling when too tall; in-game: Close, Restart + Main menu (each
  behind an "are you sure?"), Controls, then the display settings) and Controls
  (`controls_menu.rs`: both players' keys as hexagons with a key per corner,
  a single-player radio button in each center; binding a key already used
  moves it and flashes the old slot).
- **Debug keys** (not in the menu, by layout character): Space play/pause,
  `X` special apples, `D` distance grid, `1`–`9` apple food, `[`/`]` global
  speed, ↑/↓ cell size.
- **Cursor:** hidden on a key press while playing, shown again on any mouse
  movement (macOS hides it app-wide, so it must be shown explicitly).

## Snake length model (`snake/mod.rs`)

Conceptually a snake is a **float-length ribbon of material** flowing along its
trail at `speed` cells/s. `Body.segments` (a `VecDeque`) is just the stored
polyline for drawing/collision — the snake's length is **not** the segment
count. Head and tail are independent; nothing pins them to cell boundaries.

- **`length`** — *the conserved quantity*, the true length in cells (a float).
  Changed only by digestion and length changes (see below) —
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
  `food_left`, drift-free). The tail can cross segment boundaries mid-frame, so
  `Body::digest` follows its movement segment by segment, each at its own
  rate — applying one rate to the whole frame lost up to a frame's worth of
  growth per apple. Growth goes through `change_length`, so it moves the tail
  in the same frame. Dying snakes keep digesting.
- **Length changes (hunger mode):** `Grow`/`Shrink` apples push a `LengthChange`
  (an amount eased out over a duration) and apple-eating snakes have a constant
  `starvation` rate. Once the snake is all the way out, `Body::change_length`
  moves `emerged` along with `length`, so the tail moves directly: growth faster
  than the head pushes the tail *backwards*, and `advance` grows new segments
  behind the last one, along its `coming_from` — picked when that segment is
  grown, as straight on or the gentlest turn around any snake in the way
  (`way_back`; `advance` gets the occupied cells), and picked again only if
  something has moved into it since. At
  `hunger::MIN_LENGTH` the player becomes `State::Starved` (frozen like a crash,
  game over); other snakes `die()`. Apples a tail grows over are moved elsewhere
  (`relocate_covered_apples`); bad apples expire (`Apple::time_left`) and don't
  count towards the apple limit.

`advance(elapsed, board_dim)` runs head-move → emerge → length changes → digest →
pop-tail (or grow it backwards) and returns
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

1. Each `rendering::*_mesh` fn builds a `support::mesh::Mesh` (grid, border —
   drawn edge by edge for any set of cells, `region_grid_mesh`/`region_border_mesh`,
   the board being the full rectangle —
   apples, portals, distance grid, player path — the autopilot's `Plan` as one
   continuous curve, drawn cell by cell along the same centerline the
   snake ribbon is built around (`centerline_polyline`), so it turns exactly as
   a snake taking it would and, starting at the head's own fraction into its
   cell, is eaten continuously rather than a cell at a time; each leg is a step
   darker, 70% white for the one being walked down to 30% for the last) via `MeshBuilder` (lyon fill/
   stroke → chunked macroquad meshes; chunks kept under macroquad's per-draw
   10000-vert / 5000-index clamp).
2. `set_board_camera` sets a board-offset `Camera2D` once (pixel coords,
   **y-down**; `zoom.y` must be positive), then `mesh.draw()` per mesh on the
   default material. (One camera set for all board meshes — each `set_camera`
   flushes the GPU batch, so per-mesh sets were the bulk of the frame's cost.)
3. Snakes are special — drawn through a shader (below).

Draw order (default material, split around the snake): distance_grid,
hints (gradient style), grid, player_path → **snake (shaded)** → player_path
over the player's own eaten segments (which it passes through), apple, border,
hints (border style), portal → message text.

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
  The default `rainbow()` is an OkLab sweep (`oklab_gradient`) at constant
  lightness (0.62; eaten segments 0.8), each hue as saturated as the sRGB
  gamut allows at that lightness, capped at chroma 0.2 — so reds, blues and
  purples reach the cap while cyan (~0.1) and yellow (~0.135) are softer:
  constant lightness *and* constant chroma would be limited by cyan.
- **Draw** (`rendering/snake_mesh.rs` → `SnakeRender`): one `PaletteLut` per
  snake, baked as the `texture` of its shaded meshes. A snake is one mesh unless
  it crosses something: a crossing sets the crossing segment's `z_index` to the
  crossed one's ±1 (`PassOver`/`PassUnder`), and a snake is cut into pieces
  wherever its `z_index` changes (`runs`). Every snake's pieces are drawn
  together, lowest z first, equal z snake by snake and tail to head
  (`draw_order`), so a snake can be over another at one crossing and under it
  at the next. `game.rs` compiles the `SnakeMaterial` (`snake_material()`)
  lazily and calls `draw_shaded` per piece. The fragment shader samples the LUT by `uv.x`, clamped to the segment's
  own slot (`normal.xy`), and scales it by a per-vertex brightness (`normal.z`;
  1 for the body, lower for details like passability marks) (linear filtered →
  smooth gradients; hard segment boundaries stay sharp). Then it lights the
  smooth body as a round tube from straight above, by `uv.y` (across the
  width), and the round caps as hemispheres, like a pill's ends, by `normal.w`
  (`Surface`: how far along the cap, `sin φ`; −1 for flat surfaces). Lambert
  diffuse + a Blinn-Phong highlight down the middle; the constants are at the
  top of the fragment shader (`lighting!`, shared with the apples: in the
  smooth style they are balls lit the same way, `build_ball` +
  `ball_material`, from their vertex color). Passability marks and the
  hexagon style (snakes and apples) stay flat for now.

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

## Collision (`app/snake_management.rs`)

Apples are eaten by whatever cell the head is in, decided independently of snake
collisions (so a head can reach an apple and hit something in the same tick).

**The head's cell still decides what a snake runs into and what that does** —
`segments_at` + the worst `Outcome`, exactly as before. In the smooth draw style
geometry adds only a **veto**: a candidate whose drawn flesh the head does not
actually reach is dropped, so a head no longer crashes into a tail that has
already receded out of the way. This scope matters — testing "does the head touch
any flesh" instead would crash a head passing *through* an eaten segment, because
the ribbon is continuous and the head also touches that segment's `Normal`
neighbours. The hexagon style stays purely cell-based: segments fill their cell,
so there cells *are* the shape.

**Head to head**, two living snakes are settled as a pair: the one further
through its cell wins and carries on untouched, unless their `head_fraction`s
are within `HEAD_ON_TIE` (0.15), in which case both lose. A loser suffers what
its own eat mechanics say about the other's head (the player crashes, AI
snakes die; cutting a head kills the cutter).

The smooth ribbon has **constant width `side`**: a straight segment spans
`x ∈ [cos, cos+side]`, and a turn's cross-sections are radial with
`outer_radius - inner_radius == side` for every sharpness (a sharp turn is the
limiting case, `inner_radius == 0`). So the drawn flesh is *exactly* the
`side/2`-neighborhood of the centerline, and touching is a distance query. The
centerline is a G1 curve of straight lines and circular arcs; distance to an arc
is closed-form (`| |p − pivot| − r |` inside the swept angle, else the nearer end
point), so **turns are exact, not approximated by a polyline**.

`rendering/segments/centerline.rs` (`Centerline`) builds it from a `Body`,
reusing the renderer's own `segment_descriptions` and `arc_params` so the two can
never drift. Three things it must get right:

- **Both ends are shortened by one cap radius** (`cap::truncate_for_caps`, shared
  with `build_round_caps`), because that is where the round caps take over: the
  flesh reaches one radius past the *shortened* end. Measuring from the full path
  end gives every snake half a cell-side of reach it does not draw.
- **A segment can draw nothing** — a spent tail or a fresh head lies entirely
  inside its round cap, and the cap's flesh hangs off the *neighbouring*
  segment's centerline. `distance_to` falls back to the immediate neighbours; a
  cap is never longer than one radius, so that is far enough.
- **Wrapping.** A body can cross a board edge, where two adjacent cells are a
  whole board apart in board coordinates. Anything placed relative to the head
  (the cap base, a neighbour in the fallback) is reached by *stepping* with
  `board::cartesian_step` instead of reading its own position. The per-direction
  cartesian step is constant; the hex coordinate delta is not — it depends on
  column parity. Candidates themselves share the head's cell, so their own
  position is already right.

Tested in `centerline.rs`: every vertex of the real rendered ribbon sits
`side/2` from the centerline, across all 30 turn combinations.

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
- **Web colors looked duller than native** on a Mac: the native OpenGL view is
  untagged, so macOS shows its values as display-native (Display P3), while a
  WebGL canvas defaults to sRGB and gets color-managed down. `web/index.html`
  wraps `getContext` to set `drawingBufferColorSpace = "display-p3"` (where
  supported), matching native. Raw texture/vertex data isn't converted, so
  nothing else needs to change.
- **macroquad's glyph cache only grows:** every (character, font size) pair is
  rasterized into one atlas texture that doubles when full and is never
  cleared. Text sized with the window adds sizes on every resize frame until
  the atlas outgrows the GPU ("texture unloadable") and all text vanishes.
  `support::text` rasterizes at a few fixed sizes and scales with `font_scale`.
- **Intermittent head judder on the web** (all browsers, not natively): the
  game renders every frame at the right position and its per-frame work is
  tiny (≤ 2 ms measured), but some frames reach the screen a refresh late,
  so the head seems to step back and forth. It comes and goes — system
  load on the browser/compositor path (GPU contention, memory pressure,
  screen recording), not our code. Measured from screen recordings by
  tracking the head tip per frame; don't re-chase it as a pacing bug.

## TODOs / not yet done

- **Birth/death hole graphics** — the model exists (see [Snake length model](#snake-length-model-snakemodrs));
  the old black-hole circle was removed and nothing is drawn in its place yet.
  Reimplement as a hole opening/closing at the pinned end with the snake fading
  to black as it enters/leaves. Collision graphics (a crash effect) are a
  similar localized effect and want a shared approach.
- **Hunger mode follow-ups** — starving has no animation yet (the game just
  freezes at `hunger::MIN_LENGTH`, `State::Starved`).
- **Snake lighting follow-ups** (`graphics-updates`): the first pass
  (light straight from above) is in but **untuned and not yet looked at**:
  the four constants at the top of the fragment shader were picked by
  reasoning, not by eye. Then: a light from a fixed direction on screen
  (off-center highlight; needs each vertex's across direction — the vertex
  color is free for it), passability marks as part of the tube instead of a
  flat 2D overlay (they'd take the body's `uv.y` where they sit), and the
  hexagon style (flat for now, `uv.y` fixed at 0.5).
- **Perf, more generally:** each frame rebuilds the snake mesh and the border
  hints; worth trimming if frame work ever matters (it measured ≤ 2 ms in a
  release wasm build at 120 Hz).
- **Autopilot pathfinding follow-ups:** the search
  (`snake_control/pathfinder/weighted_bfs.rs`) is Dijkstra (`h = 0`); a
  *landmark* heuristic (this same search run backwards from each target) would
  make it A* — a closed-form hex distance can't work, since `wrap_around` isn't
  a lattice translation. Not needed yet: the benchmark (native release,
  2026-09) plans in ~60 µs/tick on average with one target per snake, ~120 µs
  with three; the worst ticks are 1.5–2.5 ms mid-game and 5–7 ms on the first
  tick, when every snake plans three targets at once.

## Deploy (separate, in progress)

Meant to be served at `games.gorilskij.com/hexsnake` behind a Cloudflare Worker
reverse-proxy (each game its own Pages project; the Astro site is the landing).
See `web/cf-build.sh` (Cloudflare Pages build: installs Rust, release-builds,
stages wasm) and the `pub-website`/`test-website` branches.

- **Branch flow:** work → `master` → merge into `test-website` (builds the
  test.gorilskij.com version) → merge into `pub-website` (builds the
  gorilskij.com version). Pages project `hex-snake`: production branch
  `pub-website`, previews only for `test-website`; pushes to any other branch are
  recorded as skipped deployments.
- **Pages rejects files over 25 MiB.** The release profile keeps debuginfo (for
  native profiling), so `cf-build.sh` builds with `CARGO_PROFILE_RELEASE_DEBUG=false`
  (~1 MiB wasm instead of ~25 MiB).
