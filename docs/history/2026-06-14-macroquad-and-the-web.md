# 2026-06 – 08: macroquad, the web, the ribbon

After two years untouched, the game was revived. Drawn from the commits and the notes of the
time ([working notes](working-notes-2026-10-03.md)).

- **06-14** — It compiles again (stale code fixed, the published `enum_rotate`), the start screen
  implemented properly, timing fixes.
- **06-27** — **Ported to macroquad** for native + wasm (branch `wasm`): one backend, no `cfg`
  split. A ggez-compatibility layer (`gfx/`) eased the port.
- **06-28** — Deployed as a Cloudflare Pages project (`hex-snake`) for the `/hexsnake` subpath
  behind the site's router (`games.gorilskij.com/hexsnake/`), with `pub-website` / `test-website`
  branches.
- **07-03 – 05** — **The gradient on the GPU** (branch `shaders`): it used to be faked by chopping
  each segment into ~20 flat-coloured slices on the CPU, the main cost of a frame; now one ribbon
  polygon per segment samples a LUT texture. Then the compatibility layer was dissolved into
  direct macroquad calls (`Context`, `Canvas`, keyboard input, `MeshBuilder`), `anyhow`, one
  `Color` type.
- **07-06 – 09** — Snake speed decoupled from the frame control, autopilot fixes, enum-keyed eat
  mechanics maps, fewer camera sets.
- **07-16** — Round head and tail.
- **07-21** — **The float-length model**: the tail desynchronised from the head, fractional food,
  the birth and death holes ([the snake as a ribbon](../explanation/snake-as-a-ribbon.md)).
- **08-12** — The black-hole graphics removed (death recedes into the hole with no visual; proper
  hole graphics are still to do).

Next: [modes, menus and the autopilot](2026-09-12-modes-menus-autopilot.md).
