# Backlog

Everything open, in one place. Remove an item when it is done (and record it in the
[history](history/README.md)); add one the moment it is found. Items marked **(owner)** wait on
the owner's decision. Last reviewed **2026-10-03**.

## Releasing

- **`master` is not deployed**: prod and test run the game as of 2026-09-19
  ([status](status.md#deployed-vs-master)). **(owner)** decides when to merge it into
  `test-website`.

## The game

- **Birth/death hole graphics.** The model exists ([snake model](reference/snake-model.md)); the
  old black-hole circle was removed and nothing is drawn in its place. Reimplement as a hole
  opening/closing at the pinned end, the snake fading to black as it enters/leaves. A crash
  effect is a similar localised effect and wants a shared approach.
- **Hunger mode:** starving has no animation (the game just freezes at `hunger::MIN_LENGTH`,
  `State::Starved`).
- **Snake lighting follow-ups.** The first pass (light from straight above) is in and was looked
  at once (the highlight dimmed to `SPECULAR` 0.2); the other `lighting!` constants were picked
  by reasoning, not by eye. Then: a light from a fixed direction on screen (an off-centre
  highlight; needs each vertex's across direction — the vertex colour is free for it),
  passability marks as part of the tube instead of a flat overlay (they would take the body's
  `uv.y` where they sit), and the Hexagon style (flat for now, `uv.y` fixed at 0.5).
- **Passability marks:** the head segment's mark jumps on instantly when the next segment is
  known to be eaten; easing is still to be designed.
- **Portals** (`app/portal/`) are dormant: nothing creates one. Branches `gameplay-proto` and
  `alt-portals` hold 2026-07 prototypes (dynamic wormholes, power apples). **(owner)**.

## Performance

- Each frame rebuilds the snake meshes (the light hints recompute only their 10 lights); worth
  trimming if frame work ever matters (measured ≤ 2 ms in a release wasm build at 120 Hz).
- Each snake's LUT texture is rebuilt per frame (GPU texture churn; judged not worth fixing).
- **Autopilot search** is Dijkstra (`h = 0`). A *landmark* heuristic (the same search run
  backwards from each target) would make it A* — a closed-form hex distance can't work, since
  `wrap_around` isn't a lattice translation. Not needed yet ([cost](reference/autopilot.md#cost)).

## Maintenance

- **README** (the public one): the text was brought up to date on 2026-10-03, but the
  screenshots (snipboard-hosted, from the ggez days) and the video are old. New screenshots and a
  video are needed.
- **`quad_storage` version mismatch** logged in the browser console (the vendored
  `quad-storage.js` is 0.1.4); whether preferences still save on the web is unchecked.
- **The toolchain floats**: `rust-toolchain.toml` says `nightly`, not a dated one, so every
  Cloudflare build uses whatever nightly is current — a build can break on its own. Pinning a
  date (as mandelbrot does) would make builds reproducible; the crate needs no nightly feature,
  only nightly rustfmt options. **(owner)**.
- **Old branches** (`faster-graphics`, `pub`, `round-ends-ghetto-variant`, `round-ends-proper`,
  and the prototypes above) — keep or delete. **(owner)**.

## Decided (do not re-chase)

- **The web head judder** is the browser's frame delivery, not our pacing ([pitfalls](reference/pitfalls.md#the-web)).
