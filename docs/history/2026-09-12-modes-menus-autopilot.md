# 2026-09-12 – 23: modes, menus, the autopilot

Drawn from the commits. Current facts: the [reference](../reference/README.md).

## 09-12 – 13

- Smooth frame pacing, **MSAA**; **passability marks** on eaten segments the player can pass
  through; a global speed multiplier (debug).
- **Border hints** (where the head would come out), then as recoloured border stretches with a
  toggleable style; overlapping collisions resolved by the worst outcome.
- The deployed wasm built without debug info (Pages' 25 MiB limit); **Display P3** for the web
  canvas (web colours looked duller than native); world time no longer freezes when the first
  frame reads as zero; the PRNG seeded from the wall clock; autopilot crashes fixed and its search
  parametrised (`Weights`).

## 09-16

- **Hunger mode** (`game_mode.rs`): growth at once, steady starvation, bad apples.
- **Collide with the snake as drawn**, not only its cells (the geometric veto).
- Teleport border hints; the package renamed `hex-snake`.

## 09-19

- **Preferences persist** (natively a file, on the web localStorage).
- The start screen brought back on macroquad (demos in step, palette arrows centred), then **in-game
  menus, rebindable controls and a bundled font** (branch `menus`); the options menu reordered,
  special apples and the distance grid moved to debug keys; grid and border drawn edge by edge for
  any set of cells; hexagonal start-screen demos; **keys by layout everywhere**; both players'
  controls on one screen; the cursor hidden on key presses.
- **Exact digestion** (per segment, not per frame), and the autopilot path drawn over eaten
  segments.

This state (2026-09-19) is what `test-website` and `pub-website` still run
([status](../status.md#deployed-vs-master)).

## 09-20 – 23

- **A multi-target autopilot** (plans of several legs), hints that follow the head.
- The distance grid judges other snakes by how the player eats them; stats count every polygon.
- Spawn points as their own function, with tests.
- **Head to head: the snake further through its cell wins** (ties within 0.15 both lose).
- Snakes that run out of life, and rain at the bottom, **sink into a hole**.
- The autopilot walks its survival crawl instead of redoing it every tick; **appetite**: bad apples
  avoided, not targeted; a **headless planning benchmark**.
- A tail growing backwards goes around what is in its way.
- The game mode picked on the start screen.
- **Snakes over and under each other by z**, across snakes.
- **The snake lit as a round tube with pill-shaped ends** (the highlight then dimmed, specular
  0.3 → 0.2).
- Dependencies bumped, **edition 2024**, no nightly features left; a `cargo fmt` commit, skipped
  by `git blame` (`.git-blame-ignore-revs`).
