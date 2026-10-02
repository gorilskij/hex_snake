# Controllers and the autopilot

`snake_control/`: what steers each snake. A `Template` (`snake_control/mod.rs`) describes a
controller; the snake builder turns it into a `Controller` (`next_dir` each step; `get_plan` for
autopilot-like ones, for drawing the path). Returning `None` from `next_dir` makes the snake ask
again on the next graphics frame; returning the same direction waits for the next game frame.

| template | controller | used for |
|---|---|---|
| `Keyboard { side }` | `keyboard.rs` | players (`side` = whose keys; `None` = the single player). Keeps a short queue of upcoming turns |
| `Mouse` | `mouse.rs` | a mouse-steered snake |
| `Programmed(moves)` | `programmed.rs` | the start screen's demo loops |
| `Killer` | `killer.rs` | AI snakes that hunt the player |
| `AppleSeeker { pathfinder, targets, appetite }` | `apple_seeker.rs` | competitor AI snakes and the player's **autopilot** |
| `Rain` | `rain.rs` | the "rain" special apple's falling snakes |

The player's autopilot (toggled in the menu) plans **3 targets ahead** (`autopilot_targets(3)`,
`main.rs`) with the weighted search and a space-filling backup; competitor snakes plan 1.

## Searching (`pathfinder/`)

- **`WeightedBFS`** (`weighted_bfs.rs`): cheapest-path search with costs in a `Weights` struct
  (step, blunt turn, sharp turn, teleport, pass-through). It is Dijkstra (`h = 0`).
- **`Obstacles`**: occupied cells, judged by the snake's own `eat_self` for its own segments and
  `eat_other` for other snakes'.
- **`space_filling.rs`**: the survival fallback — a crawl of up to `CRAWL_LENGTH` = 20 cells,
  each step into the free neighbour with the most room behind it.
- **`with_backup.rs`**: a search with a fallback.

A search finds **one `Leg`**: the cheapest way from a start state `(cell, heading)` to any target
not already taken, around what the plan has already `Committed` to. `Leg::target` is `None` for
the survival crawl, which is what makes a plan a fallback (it is only ever accepted as a plan's
only leg).

## Plans (`apple_seeker.rs`)

Chaining legs into a `Plan` is **path management**, done by `apple_seeker.rs`, not by the
searches:

- The head walks the plan off the front; a new leg is appended at the back as targets are
  eaten, so **the committed part of the route never moves**.
- The crawl is **walked, not recomputed**, until an `Opening` says a target may be reachable: a
  blocked cell walling in the reachable area frees up, or a new apple appears inside it.
- The route is an obstacle to its own later legs: cells it uses are `Blocked`, except targets,
  which will hold an eaten segment and are `Passable` when the snake's own eat mechanics say it
  can pass through one.
- An apple that appears **on** the route is promoted to a target by splitting that leg in two at
  its cell, leaving the route untouched; that can push a plan past its target count, and
  extending waits until it is back under.

## Appetite (`appetite.rs`)

What a search goes for, per apple kind (as `Knowledge` is per segment kind): positive values
are `Goals::targets`, negative ones `Goals::avoid` — that much extra cost to cross their cell.
The player's autopilot avoids shrink apples at −15 (`autopilot_appetite`); one landing on the
route triggers a single replan.

## Cost

The headless benchmark ([tests](testing.md)), native release, 2026-09: planning ~60 µs per tick
on average with one target per snake, ~120 µs with three; the worst ticks 1.5–2.5 ms mid-game and
5–7 ms on the first tick, when every snake plans three targets at once.
