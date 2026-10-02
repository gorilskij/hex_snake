# Build and serve the web version

How it works: [native and web](../reference/web-build.md).

```
./web/build.sh release                                     # → web/hex_snake.wasm, ~25 MB (debug info)
CARGO_PROFILE_RELEASE_DEBUG=false ./web/build.sh release   # ~2 MB, what is deployed
(cd web && python3 -m http.server 4000)                    # http://127.0.0.1:4000/index.html
```

- Use a **release** build to judge smoothness: the debug build (`./web/build.sh`) is far too slow
  in wasm.
- **Shaders can only be checked in a window**: after touching `support/material.rs`, load the
  page (or `cargo run`) and look.
- After touching randomness, check the wasm's imports: a missing `__getrandom_v03_custom` only
  fails at page load ([pitfalls](../reference/pitfalls.md#the-web)).
- Preferences in the browser are in localStorage for `127.0.0.1:4000`, separate from the deployed
  game's.

To check it like production (only `web/dist`, at `/games/hexsnake/`, through the Worker):
`bash web/cf-build.sh`, then `npx wrangler dev` and open `http://localhost:8787/games/hexsnake/`
(misses loop locally: the site's Worker, which answers them in production, is not running).
