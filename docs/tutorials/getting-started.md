# Getting started

From a fresh clone to playing, natively and in the browser.

## 1. Run it

```
git clone https://github.com/gorilskij/hex_snake.git
cd hex_snake
cargo run
```

The first build installs the nightly toolchain (`rust-toolchain.toml`). The start screen opens:
a demo snake per player looping on a little hexagonal board.

## 2. Play

- **Enter** starts a one-player Classic game. Steer with the six keys `S D F / Z X C` (up-left,
  up, up-right / down-left, down, down-right) — the letters your layout types.
- Eat apples; each leaves an **eaten segment** that travels down your body, which you can pass
  through until it is digested (it is marked with two darker lines).
- Leave the board: you come back on the side your direction leads to — each line of cells wraps
  onto itself. The red **light hints** on the border show where.
- **Space** pauses; **Esc** opens the menu (autopilot, controls, display settings, restart).
- Debug keys: `]` faster, `[` slower, `X` special apples (AI snakes, rain), `D` the distance grid.

On the start screen, the mode button switches to **Hunger** (apples grow you at once, you shrink
steadily, green bad apples shrink you), and the players button to two players (`J K L / M , .` for
the right one). All controls: [controls and screens](../reference/controls-and-screens.md).

## 3. In the browser

```
CARGO_PROFILE_RELEASE_DEBUG=false ./web/build.sh release
(cd web && python3 -m http.server 4000)
```

Open `http://127.0.0.1:4000/index.html`. (A debug wasm is far too slow to judge smoothness.)
The deployed game is at [gorilskij.com/games/hexsnake/](https://gorilskij.com/games/hexsnake/).

## 4. Next

[Architecture](../explanation/architecture.md), [the snake as a ribbon](../explanation/snake-as-a-ribbon.md),
then [pitfalls](../reference/pitfalls.md) before changing anything.
