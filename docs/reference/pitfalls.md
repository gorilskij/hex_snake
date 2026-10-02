# Pitfalls

Traps that each cost real time. The symptom, the cause, what to do.

## macroquad and graphics

- **macroquad clamps each `draw_mesh`** to 10 000 vertices / 5 000 indices and silently drops
  the overflow. `Mesh::combine` chunks under that; build meshes through it.
- **The board camera is y-down** for `draw_mesh`; `zoom.y` must be positive (negative flips the
  board — the snake appears to move the wrong way).
- **Every `set_camera` flushes the GPU batch**: set the board camera once for all board meshes.
- **lyon triangulates arbitrarily**: never feed a curved outline to lyon fill and derive
  per-vertex attributes from position — triangles cross constant-colour lines and leave seams.
  Use the explicit cross-section ribbon.
- **`atan2` wraps at ±π**: the old position-based uv hit it at the turn-start seam (avoided now:
  `frac` is generated per cross-section).
- **Shaders must be GLSL ES 1.00, WebGL1-clean** (`attribute`, `varying`, `texture2D`, precision
  qualifiers): one shader serves macOS OpenGL and WebGL. A headless sandbox can't compile them:
  check in a window or the browser.
- **macroquad's glyph cache only grows**: every (character, size) pair goes into one atlas that
  doubles when full and is never cleared. Text sized with the window added sizes on every resize
  frame until the atlas outgrew the GPU ("texture unloadable") and all text vanished.
  `support::text` rasterises at a few fixed sizes and scales (`font_scale`).
- **`Texture2D` is reference-counted** (freed on the last drop), so rebuilding each snake's LUT
  every frame does not leak — but it churns a GPU texture per snake per frame (judged not worth
  fixing).

## The web

- **std `Instant::now()` panics on wasm**: use `support::time::Instant`.
- **getrandom on wasm: the `custom` backend only** ([web build](web-build.md#the-wasm-specifics));
  never `wasm_js`, `web-time` or `instant` (they pull in wasm-bindgen, which clashes with
  macroquad's loader). A missing `__getrandom_v03_custom` fails only at page load: check the
  wasm's imports after touching it.
- **Web colours looked duller than native** until the canvas was set to Display P3.
- **Intermittent head judder on the web** (all browsers, not natively): the game renders every
  frame at the right position and its per-frame work is tiny (≤ 2 ms measured), but some frames
  reach the screen a refresh late, so the head seems to step back and forth. It comes and goes
  with load on the browser/compositor path (GPU contention, memory pressure, screen recording).
  Measured from screen recordings by tracking the head tip per frame. **Not our code: don't
  re-chase it as a pacing bug.**
- **The `quad_storage` version warning**: the console logs `Plugin quad_storage version mismatch`
  (the vendored `quad-storage.js` is 0.1.4; the crate reports another version). Present on the
  deployed version too; whether preferences still save there is unchecked ([backlog](../backlog.md)).

## The game logic

- **Never store the tail, never compute length as a difference**: the tail is derived from
  `length` and the holes ([snake model](snake-model.md)); anything else drifts.
- **Digest per segment, not per frame**: one rate for a whole frame lost up to a frame's worth of
  growth per apple.
- **Geometry may only veto a collision, not decide it**: "does the head touch any flesh" crashes a
  head passing through an eaten segment ([collision](collision.md)).
- **Across a board edge, step, don't read positions**: two adjacent cells can be a board apart in
  coordinates (`board::cartesian_step`).

## Process

- **`master` carries ongoing work and is never deployed directly.** On 2026-10-02 the test Worker
  was built from a `master`-based branch and showed unreleased work on test; deploys go through
  `test-website` / `pub-website` ([deploy](../how-to/deploy.md)).
