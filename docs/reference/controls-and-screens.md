# Controls and screens

## Keys follow the layout

Key codes are positional on macOS, Windows and the web (Linux X11 is layout-based), so
**a key that types something is identified by the character it types** (`Key::Char`,
uppercase), paired from miniquad's ordered key-down and char events; other keys keep their
code (`Key::Code`) — `app/key.rs`, `KeyInput`. Every key check, debug keys included, goes by
the layout's character. (The owner types Dvorak.)

## Movement

Six keys, one per hex direction. Defaults (`app/key.rs`), one player on each side of the
keyboard:

| direction | left player | right player |
|---|---|---|
| up-left | `S` | `J` |
| up | `D` | `K` |
| up-right | `F` | `L` |
| down-left | `Z` | `M` |
| down | `X` | `,` |
| down-right | `C` | `.` |

Bindings live in `Prefs` (left player, right player, and which side a single player uses),
edited in Options → Controls; the keyboard controller looks them up on every press. A queue
of upcoming turns (the controller's) lets quick presses chain.

## The screens

A stack (`main.rs`, `Transition`): `StartScreen` stays at the bottom; starting a game pushes a
`Game`; "Main menu" pops it, back to the start screen as it was left (`resume` reloads the
prefs, which the game may have changed).

**Start screen** (`app/screen/start_screen.rs`): buttons for players (one/two), mode
(Classic/Hunger, not saved) and Options; one demo per player (a hexagonal board one cell wider
than the snake's programmed loop, defined by a centre and a loop radius; following the snake
style, grid and border settings; grid and border built once per size), with palette arrows
under each.

| key | does |
|---|---|
| Enter | start |
| ← / → | change palette (one player only) |
| Esc | options |

**In game:**

| key | does |
|---|---|
| Esc | the menu |
| Space | play / pause; start over after a game over |

**Menus** (`menu.rs`, over a 90 % black layer; the game is paused while one is open and stays
paused after):
- **Options** (`options_menu.rs`): lines of hexagon buttons, scrolling when too tall. In game:
  Close, Restart and Main menu (each behind an "are you sure?"), Controls, then the display
  settings.
- **Controls** (`controls_menu.rs`): both players' keys as hexagons with a key per corner and a
  single-player radio button in each centre; binding a key already in use moves it and flashes
  the old slot.

Buttons (`button/`): immediate-mode polygons (`Button`, `ButtonData`: outline, inner shapes,
text spans), hit-tested on the exact outline; shared style: grey, green on hover, `ACCENT`
yellow when pressed.

## Debug keys

Not in the menus; by the layout's character (`app/screen/game.rs`):

| key | does |
|---|---|
| `[` / `]` | global speed multiplier down / up (`FpsControl`) |
| ↑ / ↓ | bigger / smaller cells |
| `1`–`9` | the food value of apples |
| `X` | special apples on/off (spawning AI snakes and rain) |
| `D` | the distance grid on/off |

## Other

- **The cursor** hides on a key press while playing and shows again on any mouse movement
  (macOS hides it app-wide, so it must be shown explicitly).
- **Text** uses the bundled DejaVu Sans (`support/text.rs`, `assets/fonts/`); macroquad's
  default font is ASCII-only.
- **Preferences** (`app/prefs.rs`, `Prefs`) are saved on every change: display settings, key
  bindings, the single-player side. Natively a file, `prefs` in the `com.gorilskij.hex-snake` config
  directory (`directories::ProjectDirs`; on macOS under `~/Library/Application Support/`); on the
  web the browser's localStorage, per origin (`support/storage.rs`). Defaults: draw style Smooth, grid and border
  on.
