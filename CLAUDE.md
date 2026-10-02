# hex_snake

Snake on a hexagonal board, in Rust on macroquad, native and in the browser
(`gorilskij.com/games/hexsnake/`). **All documentation is in [`docs/`](docs/README.md).**

## Start here

1. [`docs/status.md`](docs/status.md) — what is deployed (behind `master`), branches, test
   baselines.
2. [`docs/backlog.md`](docs/backlog.md) — everything open.
3. [`docs/reference/pitfalls.md`](docs/reference/pitfalls.md) — before touching meshes, shaders,
   the snake model, collisions or the web build.
4. Then the part of [`docs/`](docs/README.md) the task needs.

## Keep the docs true

- Every change updates `docs/` in the same change: the reference pages to the new truth, a dated
  [history](docs/history/README.md) entry with the reasons and the rejected alternatives, and
  [status](docs/status.md) / [backlog](docs/backlog.md) when they change. Findings go into the
  docs the moment they surface, not only into the reply.
- Reference pages hold no history; the history holds no current rules.
- `cargo test --test docs` must pass (links, anchors, every page reachable).
- Keep this file short and pointing into `docs/`.

## How the owner likes to work

- **`master` is ongoing work: never deploy it directly.** Deploys go `master` → `test-website` →
  `pub-website`, each on a push, when the owner says ([deploy](docs/how-to/deploy.md)).
- **Look at it**: shaders and the look can only be checked in a window or the browser — tests
  cover the logic, not the picture.
- **Keys go by the character the layout types** (the owner types Dvorak), debug keys included.
- `cargo fmt` before committing (nightly rustfmt).

## Most-used commands

```
cargo run                                                  # the game
cargo test                                                 # tests (baseline: docs/status.md)
CARGO_PROFILE_RELEASE_DEBUG=false ./web/build.sh release   # the web build → web/hex_snake.wasm
(cd web && python3 -m http.server 4000)                    # http://127.0.0.1:4000/index.html
```

Everything else: [`docs/reference/commands.md`](docs/reference/commands.md).
