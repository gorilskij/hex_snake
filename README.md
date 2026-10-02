# Hex Snake
Snake on a hexagonal board.

When you hit the wall, where you get teleported depends on the direction you were going, makes sense right?

Play it in the browser: [gorilskij.com/games/hexsnake/](https://gorilskij.com/games/hexsnake/),
or natively with `cargo run`.

One or two players, two game modes — **Classic** (apples are digested: an eaten segment travels
down your body, and you can pass through it until it is digested) and **Hunger** (apples grow you
at once, you shrink steadily, bad apples shrink you more) — AI snakes that compete for apples or
hunt you, an autopilot that plans several apples ahead, and hints on the border showing where you
would come out.

[Video Demo](https://youtu.be/REm_7UsyWT4) (an older version)

## Controls

#### Movement

Six keys, one per direction. The defaults, for one player on each side of the
keyboard (they follow the letters your layout types, not the key positions):

| Direction  | Left player | Right player |
|------------|-------------|--------------|
| Up-Left    | `S`         | `J`          |
| Up         | `D`         | `K`          |
| Up-Right   | `F`         | `L`          |
| Down-Left  | `Z`         | `M`          |
| Down       | `X`         | `,`          |
| Down-Right | `C`         | `.`          |

Keys can be rebound in Options → Controls. With one player, either side's
keys can be the ones in use.

#### Start screen

- `Enter` - Start (one or two players, Classic or Hunger mode)
- `←` / `→` - Change palette (one player)
- `Esc` - Options

#### In game

- `Esc` - Menu (restart, main menu, autopilot, controls, display settings)
- `Space` - Play / Pause, or start over after a game over

Debug keys:

- `[` / `]` - Slower / faster
- `↑` / `↓` - Bigger / smaller cells
- `1`-`9` - Nutritional value of apples
- `X` - Toggle special apples (spawning AI snakes and rain)
- `D` - Toggle the distance grid

## Screenshots

These are from an older version of the game; it looks different now.

The head of the snake is red, the tail is purple.
In Classic mode, eating an apple leaves a segment you can pass through
until it has been digested.
![](https://i.snipboard.io/jtsdXJ.jpg)

Teleportation depends on direction.
![](https://i.snipboard.io/Lu8Jac.jpg)

Sometimes you will spawn an AI snake that will
annoyingly compete for your apples (or one that will
try to kill you, or both).
![](https://i.snipboard.io/5iRPYM.jpg)

## Development

Documentation for working on the game: [`docs/`](docs/README.md).
