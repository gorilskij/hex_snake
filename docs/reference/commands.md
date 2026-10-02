# Commands

| command | does |
|---|---|
| `cargo run` | the native game (debug build; fine for iterating) |
| `cargo run --release` | the native game, optimised (with debug info, for profiling) |
| `cargo test` | the tests ([tests](testing.md)) |
| `cargo test --test docs` | the docs check |
| `cargo test --release benchmark -- --ignored --nocapture` | the headless planning benchmark |
| `cargo fmt` | format (`rustfmt.toml` uses nightly-only options) |
| `./web/build.sh` / `./web/build.sh release` | wasm build into `web/hex_snake.wasm` (debug / release) |
| `CARGO_PROFILE_RELEASE_DEBUG=false ./web/build.sh release` | the ~2 MB release wasm (what is deployed) |
| `(cd web && python3 -m http.server 4000)` | serve it: `http://127.0.0.1:4000/index.html` |
| `bash web/cf-build.sh` | what Cloudflare runs: stages `web/dist/games/hexsnake/` |
| `npx wrangler dev` | after `cf-build.sh`: the Worker locally, `http://localhost:8787/games/hexsnake/` |
| `npx wrangler deploy --dry-run` / `--env test --dry-run` | check the Worker config |
| `npx wrangler deploy` / `--env test` | deploy by hand (normally a push does it: [deploy](../how-to/deploy.md)) |

`git config blame.ignoreRevsFile .git-blame-ignore-revs` makes local `git blame` skip the
formatting commit (GitHub does it by itself).
