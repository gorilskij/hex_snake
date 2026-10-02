# Rendering

`Game::draw` (`app/screen/game.rs`) builds cached meshes lazily and draws them in z-order.
Two draw styles: **Smooth** (the default: ribbons, round caps, lit tube, balls) and
**Hexagon** (flat hexagons per cell).

## Meshes (`support/mesh.rs`)

`Mesh` is a set of macroquad meshes, each counting its polygons (for the stats overlay). The
`build_*` functions tessellate with lyon (fill/stroke) into a `Mesh`: `build_polygon`,
`build_circle`, `build_line`, `build_ball` (a circle whose uv is the position on it, for the
ball shader), and for snakes the shaded `build_shaded_polygon` / `build_shaded_ribbon`.
`Mesh::combine` merges and **chunks under macroquad's per-draw clamp** of 10 000 vertices /
5 000 indices (overflow is silently dropped otherwise). `mesh.draw()` and
`mesh.draw_shaded(material)` draw.

`set_board_camera` sets a board-offset `Camera2D` **once** for all board meshes (pixel coords,
**y-down**; `zoom.y` must be positive). Each `set_camera` flushes the GPU batch, so per-mesh
sets were the bulk of the frame's cost.

## What is drawn, in order

Each `rendering::*_mesh` function builds one mesh:

1. distance grid (debug, `D`);
2. grid (then lit by the [light hints](#light-hints));
3. player path — the autopilot's `Plan` as one continuous curve, drawn cell by cell along the
   same centreline the snake ribbon is built around (`centerline_polyline`), so it turns
   exactly as a snake taking it would and, starting at the head's own fraction into its cell,
   is eaten continuously; each leg a step darker, 70 % white for the one being walked down to
   30 % for the last;
4. **the snakes (shaded)**;
5. the player path again, over the player's own eaten segments (which it passes through);
6. apples (balls in Smooth);
7. border (then lit by the hints); grid and border are drawn edge by edge for any set of
   cells (`region_grid_mesh`, `region_border_mesh`; the board is the full rectangle);
8. portals (dormant: nothing creates one);
9. message text (`app/message.rs`: multi-line overlays, optionally on a dimmed box) and the
   stats overlay (`app/stats.rs`: polygons drawn this frame, the player's exact `length`).

## Snake colouring and drawing

- **Geometry** (`rendering/segments/`): each segment is **one polygon**, a triangle-strip
  ribbon of cross-sections (`smooth_segments::segment_cross_sections`): a straight segment is 2
  cross-sections, a turn an arc of them. Each cross-section is `(inner, outer, frac)` in default
  orientation; `point_factory::build_shaded` turns `frac` into the global body coordinate
  `uv.x = (seg_idx + (1 − frac)) / num_segments`, applies the flip/rotate/translate, and emits
  vertices through `build_shaded_ribbon` (`uv.x` = along the body, `uv.y` = across).
  - Adjacent cross-sections **share** vertices, so radial edges are single constant-`frac`
    lines: **seamless**. A sharp turn has `inner_radius == 0`: the inner point is the pivot,
    reused across cross-sections — a true single point, no hole, no seam.
  - The Hexagon style (`hexagon_segments::hexagon_outline`) is a flat hexagon with constant
    `uv.x` (`build_shaded_polygon`).
  - `rendering/segments/descriptions.rs`: `SegmentDescription`, `SegmentFraction`, turn types.
- **Colour** (`snake/palette.rs`): each frame, `build_snake_lut` samples the snake's
  `Palette` / `SegmentStyle` colour functions into a LUT texture where each segment owns a
  fixed slot of `lut_texels_per_segment` texels (segment boundaries sit on texel edges whatever
  the length). The default `rainbow()` is an **OkLab** sweep (`oklab_gradient`) at constant
  lightness 0.62 (eaten segments 0.8), each hue as saturated as the sRGB gamut allows at that
  lightness, capped at chroma 0.2 — reds, blues and purples reach the cap, cyan (~0.1) and
  yellow (~0.135) are softer (constant chroma too would be limited by cyan). `color/oklab.rs`
  has the conversions and `max_chroma` (bisection).
- **Draw** (`rendering/snake_mesh.rs`, `SnakeRender`): one `PaletteLut` per snake, the texture
  of its shaded meshes. A snake is one mesh unless it crosses something: a crossing sets the
  crossing segment's `z_index` to the crossed one's ±1 (`PassOver` / `PassUnder`), and a snake
  is cut into pieces wherever its `z_index` changes (`runs`). All snakes' pieces are drawn
  together, lowest z first, equal z snake by snake and tail to head (`draw_order`), so a snake
  can be over another at one crossing and under it at the next.
- **The shader** (`support/material.rs`, `snake_material()`, compiled lazily): samples the LUT
  by `uv.x`, clamped to the segment's own slot (`normal.xy`; linear filtering gives smooth
  gradients, segment boundaries stay sharp), scaled by a per-vertex brightness (`normal.z`: 1
  for the body, lower for details like passability marks). Then it **lights** the smooth body as
  a round tube from straight above, by `uv.y`, and the round caps as hemispheres by `normal.w`
  (`Surface`: how far along the cap, `sin φ`; −1 for flat surfaces). Lambert diffuse and a
  Blinn-Phong highlight down the middle; the constants (`AMBIENT`, `DIFFUSE`, `SPECULAR` = 0.2,
  `SHININESS`) are at the top of the shared `lighting!` snippet. Apples in Smooth are balls lit
  the same way (`build_ball` + `ball_material()`, from their vertex colour). Passability marks
  and the Hexagon style stay flat.

## Round end caps (`rendering/segments/cap.rs`)

Smooth snakes get half-circle caps at both ends. `build_round_caps` truncates the body ribbon
by one cap radius (measured **along the body path**, so caps straddle cell boundaries and bend
around turns) and fills the gap with a half-circle profile (`dist = radius·sin φ`, half-width
∝ `cos φ`). A head `Crashed` into an obstacle keeps its flat face; the radius shrinks for very
short snakes so the caps don't overlap.

## Passability marks (`rendering/segments/marks.rs`)

Eaten segments a snake can pass through get marks in a **darker shade of the segment's own
colour** (the same LUT lookup, `BRIGHTNESS` = 0.8), so the player can tell them from look-alikes
they would crash into (other snakes' eaten segments). Smooth: two lines inset from the edges
with round ends, following the edges' curvature (arcs on turns, the inner one shorter; a sharp
turn's inner line is a dot); where the neighbouring segment is marked too, the line runs flat
to the shared edge, so consecutive marked segments show one line. The head segment joins ahead
as soon as the next segment is known to be eaten (`Snake::upcoming_eaten_segment`: direction
locked in and an `Eat` apple in the next cell) — an instant jump for now, hidden because lines
are cut to the segment's drawn fraction. Hexagon: a small hexagon in the middle. Dimensions:
the consts at the top of the file (tuned by eye).

**Marked ⇒ passable, by construction:** `EatMechanics::is_marked(segment_type)` =
`mark_passable` (opt-in via `.mark_passable()`; only the player sets it) **and** the snake's own
behaviour against that segment type is inert. Not every passable segment is marked.

## Light hints

`app/light_hints.rs`, on or off: lights on the border that show the grid and border near the
head — in their own colours where they are hidden (a hidden grid is lit as lines), in red
(`palette.light_hint_color`) over them where they are drawn — through `light_material`:

- one light per side of the board, level with the head's tip on a straight line through the
  middle of that side's zigzag (the nearest point on the zigzag itself would hop from tooth to
  tooth), brighter as the head closes in from `LIGHT_RANGE` (3 cells), reaching `LIGHT_RADIUS`
  (3 cells);
- one per direction where the head would come out, as bright as where it would leave: every
  line of cells wraps onto itself, so that is where the line through the head's tip leaves the
  smoothed border going the other way (all six directions, so none pops on or off as the head
  turns).

At most `MAX_LIGHTS` = 10. Each lit mesh is drawn at its own depth with the depth test on, so
where it overlaps itself a translucent pixel is blended once, not twice. Only the 10 lights are
recomputed per frame.

## Colour on the web

The web canvas is set to **Display P3** (`web/index.html` wraps `getContext` to set
`drawingBufferColorSpace = "display-p3"` where supported): natively the OpenGL view is untagged,
so macOS shows its values as display-native (P3), while a WebGL canvas defaults to sRGB and
looked duller. Raw texture and vertex data are not converted, so nothing else changes.
