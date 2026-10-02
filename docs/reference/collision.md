# Collision (`app/snake_management.rs`)

- **Apples** are eaten by whatever cell the head is in, decided independently of snake
  collisions (a head can reach an apple and hit something in the same tick).
- **The head's cell decides what a snake runs into and what that does**: `segments_at` lists the
  segments in it, and the **worst `Outcome`** among them applies (overlapping collisions resolve
  to the worst).
- **In the Smooth style, geometry only vetoes**: a candidate whose drawn flesh the head does not
  actually reach is dropped, so a head no longer crashes into a tail that has already receded
  out of the way. That scope matters: testing "does the head touch any flesh" instead would
  crash a head passing *through* an eaten segment, because the ribbon is continuous and the head
  also touches that segment's `Normal` neighbours. The Hexagon style stays purely cell-based
  (segments fill their cell, so there cells *are* the shape).
- **Head to head**, two living snakes are settled as a pair: the one further through its cell
  wins and carries on untouched, unless their `head_fraction`s are within `HEAD_ON_TIE` (0.15),
  in which case both lose. A loser suffers what its own eat mechanics say about the other's head
  (the player crashes, AI snakes die; cutting a head kills the cutter).

## The centreline

The smooth ribbon has **constant width `side`**: a straight segment spans `x ∈ [cos, cos +
side]`, and a turn's cross-sections are radial with `outer_radius − inner_radius == side` for
every sharpness (a sharp turn is the limit, `inner_radius == 0`). So the drawn flesh is
*exactly* the `side/2`-neighbourhood of the centreline, and touching is a distance query. The
centreline is a G1 curve of straight lines and circular arcs; distance to an arc is
closed-form (`| |p − pivot| − r |` inside the swept angle, else the nearer end point), so
**turns are exact, not approximated by a polyline**.

`rendering/segments/centerline.rs` (`Centerline`) builds it from a `Body`, reusing the
renderer's own `segment_descriptions` and `arc_params`, so the two can never drift. Three things
it must get right:

- **Both ends are shortened by one cap radius** (`cap::truncate_for_caps`, shared with
  `build_round_caps`), because that is where the round caps take over: the flesh reaches one
  radius past the *shortened* end. Measuring from the full path end gives every snake half a
  cell side of reach it does not draw.
- **A segment can draw nothing** — a spent tail or a fresh head lies entirely inside its round
  cap, and the cap's flesh hangs off the *neighbouring* segment's centreline. `distance_to` falls
  back to the immediate neighbours; a cap is never longer than one radius, so that is far enough.
- **Wrapping.** A body can cross a board edge, where two adjacent cells are a whole board apart
  in board coordinates. Anything placed relative to the head (the cap base, a neighbour in the
  fallback) is reached by *stepping* with `board::cartesian_step` instead of reading its own
  position. The per-direction cartesian step is constant; the hex coordinate delta is not (it
  depends on column parity). Candidates share the head's cell, so their own position is right.

Tested in `centerline.rs`: every vertex of the real rendered ribbon sits `side/2` from the
centreline, across all 30 turn combinations.

## The board and wrapping

The board is a rectangle of hex cells (`basic/board.rs`; `HexPoint` coordinates, `Dir` the six
directions, `CellDim` the cell geometry). Leaving it wraps — and **where a snake comes back
depends on the direction it left in** (every line of cells wraps onto itself), which is the
game's signature rule. The [light hints](rendering.md#light-hints) show where.
