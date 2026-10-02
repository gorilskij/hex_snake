# 2026-09-28: the OkLab rainbow, ball apples, light hints

Branches `graphics-updates` and `light-hints`, merged into `master` the same day.

- **An OkLab constant-lightness rainbow**: each hue as saturated as the sRGB gamut allows at
  lightness 0.62 (eaten segments 0.8), capped at chroma 0.2 — constant chroma too would have been
  limited by cyan (~0.1).
- **Apples lit as balls**, like the snake (`build_ball`, `ball_material`).
- **Light hints** replace the border hints: lights on the grid and border near the head — in their
  own colours where those are hidden, in red over them where they are drawn — and lights where the
  head would come out (all six directions, so none pops on or off as the head turns), depth-tested
  so overlaps blend once. Then reduced to **one hint style, on or off**.
- `CLAUDE.md` caught up with the merged graphics work.

Not deployed: [status](../status.md#deployed-vs-master).
