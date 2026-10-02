# 2026-10-02 – 03: hosting on Workers, and these docs

## Hosting on Workers (2026-10-02)

Together with the owner's site and mandelbrot, the game moved from a Cloudflare Pages project
behind a router to a Cloudflare Worker with static assets, and from
`games.gorilskij.com/hexsnake/` to **`gorilskij.com/games/hexsnake/`** (`games.gorilskij.com` now
redirects). The full account — reasons (Pages' public `*.pages.dev` addresses bypassed the test
login), alternatives, how it was checked — is in the `gorilskij.com` repo's history
(`docs/history/2026-10-02-workers-migration.md`).

What changed here: `web/cf-build.sh` stages only the page, the vendored JS and the wasm into
`web/dist/games/hexsnake/` (under Pages the whole `web/` was published, build scripts included);
`web/worker.js` passes misses to the site's 404 page; `wrangler.toml` defines `hex-snake` and
`hex-snake-test`; Workers Builds deploys on push to `test-website` / `pub-website`. Checked in
headless Chrome from the new path: the start screen renders, identical page files to before.
Found: the `quad_storage` version warning in the console (also on the old deploy); preferences
reset once for everyone (they are per origin).

**A mistake:** the hosting commit was first made on `master` and the test Worker deployed from it,
which put `master`'s unreleased work (the lit tube, the "Classic" mode button…) on test. The owner
had not asked for that. The commit was rebased onto `test-website` and test redeployed with
`test-website`'s game code; `master` is never deployed directly
([pitfalls](../reference/pitfalls.md#process)). On 2026-10-03 `test-website` was merged back into
`master` (only the hosting files changed).

## The docs (2026-10-03)

The notes moved from `CLAUDE.md` into `docs/` (Diátaxis, plus history, status and backlog),
following the owner's other repos; the old `CLAUDE.md` is archived verbatim
([working notes](working-notes-2026-10-03.md)). Every identifier and constant it named was checked
against the code (all present; `partial_min_max` and `flip` are their own modules in `support/`,
and `light_hint_color` is a palette field). Added from the code: game modes and apples, the
controller templates, the native preferences path, the test files. Fixed: a stale comment in
`rust-toolchain.toml` (it cited `#![feature]` attributes that no longer exist and the Pages
builder). The README's text was brought up to date (the screenshots are still old).
`tests/docs.rs` checks the docs' links and that every page is reachable.
