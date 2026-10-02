# The snake as a ribbon

Facts: [the snake model](../reference/snake-model.md). History: [2026-07](../history/2026-06-14-macroquad-and-the-web.md).

## The problem with counting cells

A classic snake is a list of cells: grow by one cell, move by one cell. Smooth animation breaks
that — the head is partway into a cell, the tail partway out of one, food grows the snake by
fractions, and in Hunger mode a snake grows or shrinks continuously. Keeping "length" as the
number of segments plus some fractions led to accumulators drifting apart: tails that jumped,
growth lost per apple.

## Material flowing along a trail

So a snake is modelled as a **ribbon of material** of a given **length** (a float, in cells),
flowing along its trail at its speed. The segments are only the trail's polyline, kept for
drawing and collision.

- **Only `length` is the truth**, and it changes only additively: digestion adds exactly the
  food, a length change adds its amount. Nothing is computed as a difference of two running
  totals, so nothing drifts.
- **The tail is derived** from `length`, the head's fraction into its cell and how much of the
  snake is on the board — never stored. It cannot drift from the head.
- **Birth and death are holes**, not changes of length: material comes out of a birth hole at
  head speed (so the tail stays at the hole until the snake is all out), and a dying snake's
  head stops in a death hole while the rest drains into it. Both are counts of material
  (`emerged`, `swallowed`), independent of each other.

This made Hunger mode possible almost for free: growth faster than the head simply moves the
tail backwards (the trail is extended behind the tail, around whatever is in the way), and
shrinking moves it forwards.

## The drawn ribbon is the collision shape

The smooth ribbon has a constant width, so the drawn snake is exactly the half-width
neighbourhood of a centreline made of lines and circular arcs. That makes "does this head touch
that snake as drawn" an exact distance query — used only to veto cell-based collisions where the
drawn snake is no longer there ([collision](../reference/collision.md)).
