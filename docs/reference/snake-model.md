# The snake model (`snake/mod.rs`)

Why it is built this way: [the snake as a ribbon](../explanation/snake-as-a-ribbon.md).

A snake is a **float-length ribbon of material** flowing along its trail at `speed` cells/s.
`Body.segments` (a `VecDeque<Segment>`) is only the stored polyline for drawing and
collision — the snake's length is **not** the segment count. Head and tail are independent;
nothing pins them to cell boundaries.

## The quantities

| field | meaning |
|---|---|
| `length` | **the conserved quantity**: the true length in cells (a float). Changed only by digestion and length changes — never as a drifting difference of accumulators. Birth and death do **not** touch it; they only change how much of it is on the board |
| `head_fraction` | the head's progress into its leading cell (0..1). A new head segment is pushed at each boundary crossing (`advance_cell`) |
| `emerged`, `swallowed` | the two **holes**, in cells of material. **Birth:** material leaves the birth hole at head speed (`emerged` chases `length`), so the tail stays pinned at the hole until the snake is all out. **Death** (`state == Dying`, set by `die()`): the head pins at `HOLE_DEPTH` (0.5) into its cell and the material flowing past drains into the death hole (`swallowed` grows). The holes are independent: a snake can emerge from one while vanishing into another |
| `on_board()` | `emerged − swallowed` |

**The tail is derived, never stored:** `tail_fraction() = (visible_len − 1) + head_fraction −
on_board()`; trailing segments pop once it passes 1. So the tail cannot drift.

## One step

`advance(elapsed, board_dim)` runs head move → emerge → length changes → digest → pop the
tail (or grow it backwards), and returns whether a cell boundary was crossed (the caller then
calls `advance_cell`). A dying head never reaches a boundary, so `advance_cell` panics for
`Dying`. A snake is removed once `state == Dying && on_board() <= 0`.

## Digestion (Classic)

An `Eaten { original_food, food_left }` segment is crossed by the tail at `1/(food + 1)` speed,
growing `length` by exactly `food` (capped by `food_left`, drift-free). The tail can cross
segment boundaries mid-frame, so `Body::digest` follows its movement segment by segment, each
at its own rate (one rate for the whole frame lost up to a frame's worth of growth per apple).
Growth goes through `change_length`, so the tail moves in the same frame. Dying snakes keep
digesting.

## Length changes (Hunger)

`Grow` / `Shrink` apples push a `LengthChange` (an amount eased out over a duration), and
apple-eating snakes shrink at a constant `starvation` rate ([game modes](game-modes.md)). Once
the snake is all out, `Body::change_length` moves `emerged` along with `length`, so the tail
moves directly:

- growth faster than the head pushes the tail **backwards**, and `advance` grows new segments
  behind the last one, along its `coming_from` — picked when that segment is grown, as straight
  on or the gentlest turn around any snake in the way (`way_back`; `advance` gets the occupied
  cells), and picked again only if something has moved into it since;
- at `hunger::MIN_LENGTH` (1 cell) the player becomes `State::Starved` (frozen like a crash,
  game over); other snakes `die()`.

Apples a tail grows over are moved elsewhere (`relocate_covered_apples`).

## Segments

`Segment` carries its `SegmentType` (`Normal`, `Eaten { original_food, food_left }`,
`Crashed`), its cell (`pos`), `coming_from` (the direction to the next segment, towards the
tail), `going_to` (set for every segment but the head), `teleported` (the direction it wrapped
across the board edge in, if it did) and a `z_index` for crossings
([rendering](rendering.md#snake-colouring-and-drawing)). `snake/eat_mechanics.rs` decides what
a snake does when its head meets each segment type of itself or others (`EatBehavior`: `Cut`
the other's tail off, `Crash` (stop the game), `Die`, `PassUnder`, `PassOver` — the last two
"inert"); `snake/builder.rs` builds snakes (`SnakeBuilder`: type, palette,
controller, speed, eat mechanics, autopilot settings, starvation).

**No birth or death graphics** are drawn: the old black-hole circle was removed (2026-08);
proper hole graphics are to be reimplemented ([backlog](../backlog.md)).
