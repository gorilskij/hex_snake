# Architecture

## The shape

```
main.rs: #[macroquad::main] loop ── each frame:
   keys (layout-aware) ─▶ top Screen: key_down_event / update / draw ─▶ next_frame
                              │
        ┌─────────────────────┴───────────────────────────┐
   StartScreen (bottom of the stack)                 Game (pushed on start)
   demos, mode, options                              Environment: snakes, apples, portals,
                                                     GameContext (cell size, board, prefs,
                                                     palette, mode)
                                                     update: FpsControl time step, ticks of
                                                       ≤ ½ cell → snake_management (advance,
                                                       controllers, collisions, apples)
                                                     draw: rendering::*_mesh → Mesh → macroquad
                                                       (snakes through the lit shader)
```

- **One backend everywhere**: macroquad, natively and as wasm. No platform split in the game
  code beyond time, randomness and storage ([native and web](../reference/web-build.md)).
- **A stack of screens** with a `Screen` trait; the start screen stays underneath a game, so
  "Main menu" returns to it as it was.
- **Time-based, not tick-based**: snakes move at `speed` cells/s by the frame's duration; fast
  frames are split into ticks of at most half a cell so collisions and controllers see every
  cell.
- **The world is plain data** (`Environment`), changed by free functions in
  `snake_management.rs`; controllers only answer "which direction next".
- **Rendering builds meshes from the world** every frame (`rendering/`, cached where possible)
  and draws them with one camera. Snakes are the expensive part: a ribbon per snake, coloured by
  a texture, lit in the shader ([rendering](../reference/rendering.md)).

## Why it is like this

- **macroquad** (since 2026-06-27): the game was on ggez for years; the port (branch `wasm`) was
  made to run the same code natively and in the browser, with no `cfg` split. A
  ggez-compatibility shim eased the port and was then dissolved into direct macroquad calls.
- **Colour on the GPU** (2026-07): the gradient used to be faked by chopping each segment into
  ~20 flat-coloured slices on the CPU, re-running the arc geometry per slice — the main cost of a
  frame. A LUT texture sampled along the ribbon gives the same gradient for one polygon per
  segment.
- **A ribbon of cross-sections, not lyon fills**, for snakes: lyon triangulates curved outlines
  arbitrarily, so attributes derived from position leave seams ([pitfalls](../reference/pitfalls.md)).
- **A float-length snake** (2026-07): [the snake as a ribbon](snake-as-a-ribbon.md).
- **Collisions by cell, vetoed by geometry**: the cell rule is simple and predictable; geometry
  only removes the unfair cases where the drawn snake is already gone ([collision](../reference/collision.md)).
- **Keys by character**: a binding is the letter the key types, whatever the layout (the owner
  types Dvorak). Key codes are positional on macOS, Windows and the web, so by code "S" would be
  a different letter on each layout.
