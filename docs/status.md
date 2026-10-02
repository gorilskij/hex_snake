# Status

The living "where are we" page. Update it whenever any of this changes. Dates are absolute.
Last updated: **2026-10-03**.

## In one paragraph

The game is complete and playable natively and in the browser: one or two players, Classic and
Hunger modes, AI competitors, killers and rain, a multi-target autopilot, a start screen, in-game
menus, rebindable layout-aware controls, saved preferences, the lit-tube snake look with an OkLab
rainbow, ball apples and light hints. **`master` is well ahead of what is deployed** (see below).
Open work: [backlog](backlog.md).

## Deployed vs `master`

| | where | from | game code as of |
|---|---|---|---|
| prod | `gorilskij.com/games/hexsnake/` | `pub-website` | **2026-09-19** (`master` then: menus, controls, storage, the cursor), plus the 2026-10-02 hosting change |
| test | `test.gorilskij.com/games/hexsnake/` (behind Access) | `test-website` | the same |
| — | not deployed | `master` | 2026-09-28: everything since 2026-09-19 — multi-target autopilot and head-following hints, head-to-head by cell fraction, AI snakes sinking into holes, appetite, the planning benchmark, tails growing around obstacles, the mode picker, z-ordered crossings, the lit tube, the OkLab rainbow, ball apples, light hints, edition 2024 |

**Releasing `master` is the owner's call** ([deploy](how-to/deploy.md)): it is ongoing work. Until
the hosting change was merged back (2026-10-03) `master` lacked `wrangler.toml` and the build
changes; it has them now.

## Branches

| branch | state (2026-10-03) |
|---|---|
| `master` | the working branch and GitHub's default |
| `test-website`, `pub-website` | deploy branches |
| `autopilot-improvements`, `exact-digestion`, `game-modes`, `gameplay-improvements`, `graphics-updates`, `hints`, `light-hints`, `menus`, `shaders`, `snake-speed`, `storage`, `todo-backlog`, `dev`, `round-ends` | merged, kept as pointers |
| `faster-graphics` (16 commits, 2024-05, "checkpoint, crashing"), `gameplay-proto` (3, 2026-07: dynamic wormholes, power apples), `alt-portals` (2, 2026-07), `pub` (2, 2026-06, the old `pub` before `pub-website`), `round-ends-ghetto-variant`, `round-ends-proper` (1 each) | **unmerged** experiments |
| `workers` | the 2026-10-02 hosting change, merged; local only |

The owner's main checkout (`~/code/rust/hex_snake`) is on `light-hints`, which equals `master`'s
state of 2026-09-28.

## Tests

`cargo test` (2026-10-03): **69 passed, 1 ignored** (the benchmark), plus the docs check.

## Uncommitted work

None (2026-10-03).
