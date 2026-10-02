# 2019-12 – 2024-04: the ggez years

Drawn from the commits (mostly short messages); reasons recorded where known.

## 2019 – 2020: the game

- **2019-12-13** — "basic setup", "snake works": a hex-grid snake on **ggez**.
- **2020-02** — Teleportation fixed (the signature rule: where you come back depends on the
  direction you left in), apples, collision, the grid drawn as zigzag lines (much faster), and
  **multiplayer** (several snakes, each with its own controls).
- **2020-03 – 04** — Dvorak and QWERTY key sets for a left and a right player, a dark theme
  (`Theme` / `Palette`), multiple simultaneous crash sites, the README with screenshots,
  `rustfmt.toml`.
- **2020-11 – 12** — A control queue (no turning back into yourself), head-to-head collisions,
  fewer draw calls, teleportation indicators (half hexagons), a grid toggle, window resizing,
  O(1) apple spawning, messages on screen, screens (the start of a start screen with a demo
  snake), frame-rate mechanisms.

## 2021: AI, smooth snakes, colour

- **01** — The first AI, then evil snakes spawned by special apples, a killer AI, per-snake
  **eat mechanics** (cut, die, …), snake death; then **smooth segments**: straight and turning
  segments drawn as animated shapes (soft and sharp turns, a thin head and tail), 12-direction
  control (tried), separate game and graphics frame rates, an **OkLab** colour profile and
  skins, round turns, the autopilot toggle.
- **02 – 06** — Apple spawn strategies, `Dir12`, draw styles (hexagon, smooth), a competitor AI,
  **A\*** search (05-25), rainbow gradients (subsegments, resolution by length), mesh caching,
  debug stats.
- **09** — ggez 0.6; partial-rotation graphics; the **start screen** (infinity and figure-8 demo
  snakes, cutting); collision code factored out; **keyboard layout translation** (09-07); a debug
  scenario; black-hole birth/death animations; `lyon_geom::Arc`; a mouse controller.

## 2022 – 2024

- **2022** — ggez 0.7; snakes passing under eaten segments; snake speed; rain; the A* AI passing
  through eaten segments selectively (`PassthroughKnowledge`); a smooth RGB gradient with
  transparency; the **distance grid**; a guidance path for the player; the project flattened;
  pathfinding factored into controllers; a first round head ("very shitty prototype"); targets;
  a dot grid; ggez 0.8.
- **2023** — Round head working going straight, apples resized, passing over oneself instead of
  under, a boost, cursor hiding, grid and border toggled separately, `Environment` as a struct.
- **2024-02 – 04** — Buttons (an MVP, then factored out and rotatable), autopilot fixes,
  **portals**: teleportation cells, inverse cells, portals on the board's edges, dynamic portals
  (merged 04-16; dormant today — nothing creates one).

Next: [macroquad and the web](2026-06-14-macroquad-and-the-web.md).
