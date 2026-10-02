# Repository layout

GitHub: `gorilskij/hex_snake` (public). Default branch `master`. The crate is `hex-snake`
(binary), edition 2024.

```
CLAUDE.md, README.md          rules for Claude; the public readme (controls, old screenshots)
docs/                         this documentation
Cargo.toml, Cargo.lock        dependencies: macroquad, lyon, rand/getrandom, …
rust-toolchain.toml           nightly (unpinned) + the wasm target
rustfmt.toml                  nightly-only rustfmt options
.cargo/config.toml            the wasm target's linker flag and getrandom backend
.git-blame-ignore-revs        formatting-only commits skipped by git blame
wrangler.toml                 the Cloudflare Workers (prod and [env.test])
assets/fonts/                 DejaVu Sans (+ licence), bundled for all text
web/                          the web page and build scripts — reference/web-build.md
src/
  main.rs                     the loop, the screen stack, window config, player seeds, wasm getrandom
  app/                        game orchestration
    screen/                   the Screen trait, Transition, Environment; game.rs (the Game: world,
                              FpsControl, meshes, menu, draw/update), start_screen.rs, menu.rs,
                              options_menu.rs, controls_menu.rs, board_dim.rs
    key.rs                    layout-aware keys and player bindings
    fps_control.rs            play/pause/game over, the time step, the debug speed
    game_context.rs           GameContext: cell_dim, board_dim, prefs, palette, mode
    game_mode.rs              Classic / Hunger and the hunger tuning
    prefs.rs                  Prefs, saved on every change
    snake_management.rs       advance, spawn, collisions
    light_hints.rs            the edge hints
    palette.rs                board and background colours (not the snakes')
    message.rs, stats.rs      text overlays; the stats overlay
    distance_grid.rs          the debug distance grid
    portal/                   portals (dormant: nothing creates one)
    benchmark.rs              the headless planning benchmark (test only)
  snake/                      the snake model (mod.rs), builder.rs, eat_mechanics.rs, palette.rs (colouring, the LUT)
  snake_control/              controllers: keyboard, mouse, programmed, killer, rain, apple_seeker, appetite, pathfinder/
  apple/                      apple types (mod.rs), spawning (spawn.rs)
  rendering/                  world → meshes: grid, border, apples, portals, distance grid, player path, snakes
    segments/                 ribbons, cross-sections, caps, marks, centreline, descriptions
    shape/                    shared shapes (hexagons, arrows…) for buttons, menus, apples and hints
  basic/                      Point, HexPoint, Dir (the six directions), CellDim, board.rs
  color/                      Color (over macroquad's), oklab.rs, to_color.rs (HSL)
  view/                       borrowing helpers for "the other snakes" (split_snakes)
  button/                     immediate-mode polygon buttons and their style
  support/                    mesh.rs (Mesh, build_*, the camera), material.rs (shaders),
                              text.rs, storage.rs, time.rs, flip.rs, partial_min_max.rs
tests/docs.rs                 the docs check
```

`support/` is what remains of a ggez-compatibility layer (`gfx/`) from the port to macroquad;
the game now calls macroquad directly.

Not in git: `target/`, `web/hex_snake.wasm`, `web/dist/`, `.wrangler/`, `/profiling`, `*.svg`
(flamegraphs), `.idea/`.
