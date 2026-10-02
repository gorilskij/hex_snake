# Tests

Baseline (2026-10-03, M2 Pro): `cargo test` — **69 passed, 1 ignored** (the benchmark), plus
the docs check (`cargo test --test docs`).

Unit tests live next to the code (`#[cfg(test)]`), in: `rendering/segments/centerline.rs`
(every vertex of the real rendered ribbon sits `side/2` from the centreline, across all 30 turn
combinations), `snake/mod.rs` (digestion grows by exactly the food; a tail backing up goes straight or around
what is in the way),
`snake/palette.rs` (the OkLab rainbow), `color/oklab.rs`, `app/snake_management.rs` (head to head,
running out of life, rain at the bottom), `app/key.rs` (characters, default bindings, rebinding),
`app/prefs.rs`, `app/light_hints.rs`, `app/distance_grid.rs`, `app/screen/game.rs`,
`app/screen/start_screen.rs`, `snake_control/` (`apple_seeker.rs`, `appetite.rs`,
`programmed.rs`, `pathfinder/weighted_bfs.rs`, `pathfinder/space_filling.rs`),
`rendering/snake_mesh.rs` (draw order), `rendering/shape/collisions.rs`, and `basic/`
(`board.rs`, `dir.rs`, `hex_point.rs`).

**The benchmark** (`app/benchmark.rs`, ignored): the headless world update with 24 apple-seeking
snakes on an 80 × 50 board with 80 apples, timing planning apart from the rest
(`cargo test --release benchmark -- --ignored --nocapture`). Results of 2026-09:
[autopilot](autopilot.md#cost).

**What tests cannot cover:** shaders need a GL window, so they are checked by running the game
(natively or in the browser), and the look is judged by eye.
