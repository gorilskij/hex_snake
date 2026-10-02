# Deploy

The game is served at `gorilskij.com/games/hexsnake/` by the Cloudflare Worker `hex-snake`, and at
`test.gorilskij.com/games/hexsnake/` (behind Cloudflare Access) by `hex-snake-test`. Both deploy
themselves when their branch is pushed (Workers Builds). The whole hosting picture, shared with the
owner's site and mandelbrot, is in the `gorilskij.com` repo (`docs/reference/hosting.md`); the
games page there links here.

## The flow

```
master ──merge──▶ test-website ──merge──▶ pub-website
                   │                       │
                   ▼                       ▼
  test.gorilskij.com/games/hexsnake/   gorilskij.com/games/hexsnake/
```

1. Work on `master`. **`master` carries ongoing work and is never deployed directly** — merging
   it into `test-website` is the owner's decision to release what is on it.
2. **Test:** merge `master` into `test-website` ("Merge branch 'master' into test-website"), push.
   Workers Builds runs `bash web/cf-build.sh` then `npx wrangler deploy --env test` (~1.5 min).
3. Check it on `test.gorilskij.com/games/hexsnake/` (the owner's login).
4. **Prod:** merge `test-website` into `pub-website` ("Merge branch 'test-website' into
   pub-website"), push: `npx wrangler deploy`.

Builds of all the owner's projects queue one at a time; see them in the Cloudflare dashboard
(Workers & Pages → `hex-snake` → Deployments).

## What is deployed

- `wrangler.toml`: Worker `hex-snake`, routes `gorilskij.com/games/hexsnake` and
  `gorilskij.com/games/hexsnake/*` (zone `gorilskij.com`); `[env.test]`: `hex-snake-test` on the
  same paths of `test.gorilskij.com`. Both `workers_dev = false`, `preview_urls = false` (no
  address that bypasses the login); `name`, `routes`, `workers_dev`, `preview_urls` are restated
  under `[env.test]` because they do not inherit.
- Static assets: `web/dist`, staged by `cf-build.sh` under `games/hexsnake/` — the page, the
  vendored JS and the wasm only (not the build scripts).
- `web/worker.js` runs only when no file matches and passes the request on to the site's Worker,
  which answers with its 404 page.
- The wasm is built without debug info (static assets reject files over 25 MiB).
- **Preferences are per origin** (localStorage): the move from `games.gorilskij.com` (until
  2026-10-02) to `gorilskij.com` started everyone with default preferences once.

## By hand

```
bash web/cf-build.sh && npx wrangler deploy --env test    # test
bash web/cf-build.sh && npx wrangler deploy               # prod
```

(`npx wrangler login` once.) Replaced by the next push to that branch. Check a config change first
with `npx wrangler deploy --dry-run` (and `--env test`).
