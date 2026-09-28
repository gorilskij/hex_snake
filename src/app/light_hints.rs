//! Hints at wrapping around the board's edges: the grid and border lit up
//! near the head, both where it would leave the board and where it would
//! come back in (see [`border_lights`]).

use crate::app::game_context::GameContext;
use crate::basic::{board, CellDim, Dir, HexDim, HexPoint, Point};
use crate::rendering::segments::centerline::Centerline;
use crate::rendering::shape::{Hexagon, Shape};
use crate::snake::Body;

/// How close to a side of the board the head has to be for its light to come
/// on at all, in cell heights.
const LIGHT_RANGE: f32 = 3.;

/// How far a light reaches from its center, in cell heights.
pub const LIGHT_RADIUS: f32 = 3.;

/// A light on the border, for [`light_material`].
///
/// [`light_material`]: crate::support::material::light_material
#[derive(Copy, Clone, Debug)]
pub struct Light {
    pub pos: Point,
    /// 0 (off) to 1
    pub intensity: f32,
}

/// How many lights [`border_lights`] gives: one per side of the board, and one
/// per direction where the head would come out.
pub const LIGHTS: usize = 4 + 6;

/// Every light on the border: one per side of the board by the head (see
/// [`lights_at`]), and one per direction where the head would come out if it
/// wrapped going that way (see [`teleport_lights_at`]).
pub fn border_lights(body: &Body, gtx: &GameContext) -> [Light; LIGHTS] {
    let (board_dim, cell_dim) = (gtx.board_dim, gtx.cell_dim);
    let head = Centerline::of(body, gtx)
        .head_tip()
        .unwrap_or_else(|| body.segments[0].pos.to_cartesian(cell_dim) + Hexagon::center(cell_dim));
    let sides = Sides::of(board_dim, cell_dim);
    let mut lights = [Light { pos: head, intensity: 0. }; LIGHTS];
    lights[..4].copy_from_slice(&lights_at(head, sides, cell_dim));
    lights[4..].copy_from_slice(&teleport_lights_at(head, sides, cell_dim));
    lights
}

/// How bright a light is with the head `distance` away from where it would
/// leave the board: off from [`LIGHT_RANGE`] cells away, full on the border.
fn intensity(distance: f32, cell_dim: CellDim) -> f32 {
    let t = (1. - distance / (LIGHT_RANGE * cell_dim.height())).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

/// The board's border, smoothed out for the lights.
///
/// Each side of the border zigzags, so the point on it nearest the head (or
/// where a ray from the head hits it) hops from one tooth to the next as the
/// head slides along, and a light put there with it. Instead, each side is
/// taken as the straight line through the middle of its zigzag, which makes
/// the border a rectangle: every point on it moves exactly as the head does.
#[derive(Copy, Clone, Debug)]
struct Sides {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl Sides {
    fn of(board_dim: HexDim, cell_dim: CellDim) -> Self {
        let (mut min, mut max) = (
            Point::from((f32::INFINITY, f32::INFINITY)),
            Point::from((-f32::INFINITY, -f32::INFINITY)),
        );
        for (a, b, _) in border_segments(board_dim, cell_dim) {
            for p in [a, b] {
                (min.x, min.y) = (min.x.min(p.x), min.y.min(p.y));
                (max.x, max.y) = (max.x.max(p.x), max.y.max(p.y));
            }
        }
        // the top and bottom zigzag by half a cell (`sin`) between neighbouring
        // columns, the left and right by `cos` between a corner and a side
        Self {
            top: min.y + cell_dim.sin / 2.,
            bottom: max.y - cell_dim.sin / 2.,
            left: min.x + cell_dim.cos / 2.,
            right: max.x - cell_dim.cos / 2.,
        }
    }

    /// The nearest point inside (a head can be up to half a zigzag outside).
    fn clamp(self, p: Point) -> Point {
        Point::from((p.x.clamp(self.left, self.right), p.y.clamp(self.top, self.bottom)))
    }

    /// Where a ray from `from` (inside) going `step` leaves the rectangle.
    fn exit(self, from: Point, step: Point) -> Point {
        let along = |from: f32, step: f32, lo: f32, hi: f32| {
            if step > 1e-6 {
                (hi - from) / step
            } else if step < -1e-6 {
                (lo - from) / step
            } else {
                f32::INFINITY
            }
        };
        let t = along(from.x, step.x, self.left, self.right).min(along(from.y, step.y, self.top, self.bottom));
        from + step * t.max(0.)
    }
}

/// One light per side of the board (top, bottom, left, right), each on that
/// side's border level with `head`, and brighter the closer the head is.
///
/// A light per side rather than one on the nearest border point, so a light
/// never jumps: going into a corner, the second one fades in beside the first.
fn lights_at(head: Point, sides: Sides, cell_dim: CellDim) -> [Light; 4] {
    let Point { x, y } = sides.clamp(head);
    [
        (Point::from((x, sides.top)), (head.y - sides.top).abs()),
        (Point::from((x, sides.bottom)), (head.y - sides.bottom).abs()),
        (Point::from((sides.left, y)), (head.x - sides.left).abs()),
        (Point::from((sides.right, y)), (head.x - sides.right).abs()),
    ]
    .map(|(pos, distance)| Light {
        pos,
        intensity: intensity(distance, cell_dim),
    })
}

/// One light per direction, where the head would come out if it kept going
/// that way and wrapped around, as bright as the light where it would leave.
///
/// Every line of cells wraps onto itself — a column top to bottom, a diagonal
/// from one end to the other — so going one way, the head comes back in where
/// the line through it leaves the board going the other way. Measured on the
/// smoothed border, like the lights by the head, so these slide with it too.
///
/// All six directions, the one behind the head included, so a light never
/// pops on or off as the head turns.
fn teleport_lights_at(head: Point, sides: Sides, cell_dim: CellDim) -> [Light; 6] {
    let from = sides.clamp(head);
    let mut lights = [Light { pos: from, intensity: 0. }; 6];
    for (light, dir) in lights.iter_mut().zip(Dir::iter()) {
        let step = board::cartesian_step(dir, cell_dim);
        let leaves = sides.exit(from, step);
        *light = Light {
            pos: sides.exit(from, step * -1.),
            intensity: intensity((leaves - head).magnitude(), cell_dim),
        };
    }
    lights
}

/// Every side of the board's edge, as a segment and the way out through it.
/// These are exactly the sides that wrap: a side is on the edge precisely when
/// what lies across it is on the other side of the board.
fn border_segments(board_dim: HexDim, cell_dim: CellDim) -> Vec<(Point, Point, Dir)> {
    let corners = Hexagon::raw_points(cell_dim);
    wrap_edges(board_dim)
        .map(|(pos, dir, _)| {
            // side i runs from corner i to corner i + 1 and faces Dir i
            let origin = pos.to_cartesian(cell_dim);
            let side = dir as usize;
            (origin + corners[side], origin + corners[(side + 1) % 6], dir)
        })
        .collect()
}

/// Every edge of a cell that wraps around the board, as `(cell, direction,
/// the cell it wraps to)`.
fn wrap_edges(board_dim: HexDim) -> impl Iterator<Item = (HexPoint, Dir, HexPoint)> {
    // a single step changes h and v by at most 1, so only the outermost rows and
    // columns can leave the board
    let on_border =
        move |pos: &HexPoint| pos.h == 0 || pos.v == 0 || pos.h == board_dim.h - 1 || pos.v == board_dim.v - 1;
    (0..board_dim.h)
        .flat_map(move |h| (0..board_dim.v).map(move |v| HexPoint { h, v }))
        .filter(on_border)
        .flat_map(move |pos| Dir::iter().filter_map(move |dir| Some((pos, dir, pos.wrap_destination(dir, board_dim)?))))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teleport hints are drawn on `(destination, -dir)`, which is only a
    /// border edge — and only meets the border cleanly — if wrapping is an
    /// involution: stepping back out of where you landed must leave the board
    /// again, and land back where you came from.
    #[test]
    fn wrapping_is_symmetric() {
        for board_dim in [
            HexPoint { h: 5, v: 5 },
            HexPoint { h: 10, v: 10 },
            HexPoint { h: 20, v: 13 },
            HexPoint { h: 7, v: 20 },
        ] {
            for (pos, dir, destination) in wrap_edges(board_dim) {
                let back = destination.wrap_destination(-dir, board_dim);
                assert_eq!(
                    back,
                    Some(pos),
                    "{board_dim:?}: {pos:?} {dir:?} -> {destination:?}, but back out is {back:?}",
                );
            }
        }
    }

    const BOARD: HexDim = HexPoint { h: 20, v: 20 };
    const CELL_DIM: CellDim = CellDim { side: 50., sin: 43.30127, cos: 25. };

    fn cell_center(cell: HexPoint) -> Point {
        cell.to_cartesian(CELL_DIM) + Hexagon::center(CELL_DIM)
    }

    #[test]
    fn a_light_brightens_on_the_border_nearest_the_head() {
        let far = lights_at(
            cell_center(HexPoint { h: 10, v: 10 }),
            Sides::of(BOARD, CELL_DIM),
            CELL_DIM,
        );
        assert!(
            far.iter().all(|light| light.intensity == 0.),
            "the middle of the board is dark"
        );

        // a column up from the bottom-left corner, left of the middle: only the
        // left side's light is on, and more so the closer the head is
        let intensity = |h| lights_at(cell_center(HexPoint { h, v: 10 }), Sides::of(BOARD, CELL_DIM), CELL_DIM)[2];
        let (near, nearer) = (intensity(2), intensity(0));
        assert!(
            0. < near.intensity && near.intensity < nearer.intensity,
            "{near:?} {nearer:?}"
        );
        assert!(nearer.intensity > 0.9, "{nearer:?}");
        assert_eq!(intensity(3).intensity, 0., "out of range");
        // on the border, level with the head
        assert!(
            (near.pos.y - cell_center(HexPoint { h: 2, v: 10 }).y).abs() < 1e-3,
            "{near:?}"
        );
        assert!(
            near.pos.x < cell_center(HexPoint { h: 0, v: 10 }).x - CELL_DIM.cos,
            "{near:?}"
        );
        let lights = lights_at(
            cell_center(HexPoint { h: 0, v: 10 }),
            Sides::of(BOARD, CELL_DIM),
            CELL_DIM,
        );
        for side in [0, 1, 3] {
            assert_eq!(lights[side].intensity, 0., "side {side} is out of range");
        }
    }

    /// Sliding along the border a little at a time, the light keeps up with the
    /// head without ever jumping: it never moves more than the head does, and
    /// its brightness stays put (the head stays level with the border).
    #[test]
    fn a_light_moves_smoothly_with_the_head() {
        let start = cell_center(HexPoint { h: 2, v: 0 });
        let step = 1.;
        let mut last = lights_at(start, Sides::of(BOARD, CELL_DIM), CELL_DIM)[0];
        assert!(last.intensity > 0., "{last:?}");
        for k in 1..300 {
            let head = start + Point::from((k as f32 * step, 0.));
            let light = lights_at(head, Sides::of(BOARD, CELL_DIM), CELL_DIM)[0];
            assert!(
                (light.pos - last.pos).magnitude() <= step + 1e-3,
                "{last:?} -> {light:?}"
            );
            assert!((light.intensity - last.intensity).abs() < 1e-6, "{last:?} -> {light:?}");
            last = light;
        }
    }

    /// Near the top, the light where going up comes out is at the bottom,
    /// straight below the head, and as bright as the head's own light at the
    /// top; nothing else is on.
    #[test]
    fn going_up_comes_out_at_the_bottom() {
        let sides = Sides::of(BOARD, CELL_DIM);
        let head = cell_center(HexPoint { h: 10, v: 0 });
        let lights = teleport_lights_at(head, sides, CELL_DIM);
        let up = lights[0];
        assert!((up.pos.x - head.x).abs() < 1e-3, "{up:?}");
        assert!((up.pos.y - sides.bottom).abs() < 1e-3, "{up:?}");
        let own = lights_at(head, sides, CELL_DIM)[0];
        assert!((up.intensity - own.intensity).abs() < 1e-3, "{up:?} vs {own:?}");
        assert!(up.intensity > 0.5, "{up:?}");

        let middle = teleport_lights_at(cell_center(HexPoint { h: 10, v: 10 }), sides, CELL_DIM);
        assert!(middle.iter().all(|light| light.intensity == 0.), "{middle:?}");
    }

    /// Wherever the head slides, even through a corner, the teleport lights
    /// move no faster than it does (a diagonal's far end moves by at most the
    /// head's step over the sine of its angle with the side, twice) and their
    /// brightness never jumps.
    #[test]
    fn teleport_lights_move_smoothly_with_the_head() {
        let sides = Sides::of(BOARD, CELL_DIM);
        let start = cell_center(HexPoint { h: 1, v: 1 });
        let step = Point::from((0.5, 0.3));
        let mut last = teleport_lights_at(start, sides, CELL_DIM);
        for k in 1..2000 {
            let head = start + step * k as f32;
            let lights = teleport_lights_at(head, sides, CELL_DIM);
            for (light, last) in lights.iter().zip(&last) {
                assert!(
                    (light.pos - last.pos).magnitude() < 5. * step.magnitude(),
                    "{last:?} -> {light:?}"
                );
                assert!((light.intensity - last.intensity).abs() < 0.02, "{last:?} -> {light:?}");
            }
            last = lights;
        }
    }
}
