//! The curve a smooth-style snake's ribbon is drawn around.
//!
//! The ribbon has **constant width `side`**: a straight segment spans
//! `x ∈ [cos, cos + side]`, and a turn's cross-sections are radial, spanning
//! `[inner_radius, outer_radius]` with `outer - inner == side` for every turn
//! sharpness (a sharp turn is the limiting case, `inner_radius == 0`). So the
//! drawn flesh is exactly the set of points within `side / 2` of this curve,
//! and "do these two snakes touch" is a distance query against it rather than
//! a comparison of cells.
//!
//! Both ends are shortened by one cap radius, because that is where the round
//! caps take over (see [`cap`](super::cap)): the flesh reaches one radius past
//! the shortened end, not past the full one. Measuring from the full end would
//! give every snake half a cell-side of reach it does not draw.

use std::f32::consts::PI;

use crate::app::game_context::GameContext;
use crate::basic::board::cartesian_step;
use crate::basic::{CellDim, Point};
use crate::rendering::segments::cap::truncate_for_caps;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction};
use crate::rendering::segments::smooth_segments::{arc_params, cross_section_at};
use crate::rendering::snake_mesh::segment_descriptions;
use crate::snake::Body;

/// A snake's drawn centerline, one entry per segment (head → tail).
pub struct Centerline {
    segments: Vec<SegmentDescription>,
    cell_dim: CellDim,
    /// Half the ribbon's width: how far the flesh reaches from the centerline.
    pub half_width: f32,
    /// The round cap's radius, equal to `half_width` except on snakes too short
    /// to fit two full caps.
    pub cap_radius: f32,
}

impl Centerline {
    pub fn of(body: &Body, gtx: &GameContext) -> Self {
        let mut segments = segment_descriptions(body, gtx);
        let cap_radius = if segments.is_empty() {
            0.
        } else {
            truncate_for_caps(&mut segments)
        };

        Self {
            cell_dim: gtx.cell_dim,
            half_width: gtx.cell_dim.side / 2.,
            cap_radius,
            segments,
        }
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// The centre of the head's round cap: where the ribbon ends and the cap
    /// begins, and the point a head-on collision is measured from. `None` if
    /// the body draws nothing at all.
    ///
    /// The base can sit a segment or two back when the cap straddles a cell
    /// boundary, so this steps from the head's own cell rather than reading
    /// that segment's board position — which would be a board away if the body
    /// happens to cross a board edge there.
    pub fn head_base(&self) -> Option<Point> {
        let mut destination = self.segments.first()?.destination;
        for desc in &self.segments {
            if drawn_extent(desc.fraction) > 0. {
                let mut desc = desc.clone();
                desc.destination = destination;
                return Some(centerline_at(&desc, desc.fraction.end));
            }
            destination += cartesian_step(desc.turn.coming_from, self.cell_dim);
        }
        None
    }

    /// Distance from `probe` to the flesh of segment `idx`, which must be in
    /// the same cell as the probing head. `None` if there is nothing there to
    /// touch.
    pub fn distance_to(&self, idx: usize, probe: Point) -> Option<f32> {
        let origin = self.segments[idx].destination;
        if let Some(distance) = self.distance_at(idx, origin, probe) {
            return Some(distance);
        }

        // The segment draws nothing: a spent tail or a fresh head lies entirely
        // inside its round cap, and the cap's flesh hangs off the neighbouring
        // segment's centerline instead. A cap is never longer than one radius,
        // so the immediate neighbours are as far as this has to look.
        let before = (idx > 0)
            .then(|| self.distance_at(idx - 1, self.step_to(idx, idx - 1, origin), probe))
            .flatten();
        let after = (idx + 1 < self.len())
            .then(|| self.distance_at(idx + 1, self.step_to(idx, idx + 1, origin), probe))
            .flatten();

        match (before, after) {
            (Some(before), Some(after)) => Some(before.min(after)),
            (before, after) => before.or(after),
        }
    }

    fn distance_at(&self, idx: usize, destination: Point, probe: Point) -> Option<f32> {
        let desc = &self.segments[idx];
        if drawn_extent(desc.fraction) <= 0. {
            return None;
        }
        let mut desc = desc.clone();
        desc.destination = destination;
        Some(distance_to_centerline(&desc, desc.fraction, probe))
    }

    /// Where neighbouring segment `to` sits, given that `from` is at `origin`.
    /// `coming_from` points from a segment towards the tail.
    fn step_to(&self, from: usize, to: usize, origin: Point) -> Point {
        if to > from {
            origin + cartesian_step(self.segments[from].turn.coming_from, self.cell_dim)
        } else {
            origin - cartesian_step(self.segments[to].turn.coming_from, self.cell_dim)
        }
    }
}

fn drawn_extent(fraction: SegmentFraction) -> f32 {
    (fraction.end - fraction.start).max(0.)
}

/// The centerline point at `frac`, in board space. The ribbon is centred
/// between its two edges, so this is just their midpoint.
fn centerline_at(desc: &SegmentDescription, frac: f32) -> Point {
    let (inner, outer) = cross_section_at(desc, frac);
    desc.board_transform()((inner + outer) * 0.5)
}

/// Distance from `probe` to the part of `desc`'s centerline inside `fraction`.
///
/// Done in the segment's default orientation (the board transform is an
/// isometry, so the distance is the same), where the centerline is either a
/// vertical line or an arc around the turn's pivot.
fn distance_to_centerline(desc: &SegmentDescription, fraction: SegmentFraction, probe: Point) -> f32 {
    let q = desc.inverse_board_transform()(probe);

    // the same branch the renderer takes: a turn too shallow to curve has no
    // arc and is drawn as a straight box
    match arc_params(desc) {
        Some(arc) => {
            let radius = (arc.inner_radius + arc.outer_radius) / 2.;

            // angle around the pivot is PI at fraction 0, decreasing to
            // PI - total_angle at fraction 1
            let theta_of = |frac: f32| PI - frac * arc.total_angle;
            let (lo, hi) = (theta_of(fraction.end), theta_of(fraction.start));

            let offset = q - arc.pivot;
            let angle = offset.y.atan2(offset.x);
            if (lo..=hi).contains(&angle) {
                (offset.magnitude() - radius).abs()
            } else {
                // beyond either end of the arc: nearest of the two end points.
                // The arc lies in the upper half plane and spans at most 120°,
                // so `angle` never wraps into the range from outside it.
                let point_at = |theta: f32| {
                    let (sin, cos) = theta.sin_cos();
                    arc.pivot + Point { x: radius * cos, y: radius * sin }
                };
                let to_lo = (q - point_at(lo)).magnitude();
                let to_hi = (q - point_at(hi)).magnitude();
                to_lo.min(to_hi)
            }
        }
        None => {
            let height = desc.cell_dim.height();
            let x = desc.cell_dim.width() / 2.;
            let y = q.y.clamp(fraction.start * height, fraction.end * height);
            (q - Point { x, y }).magnitude()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic::{Dir, HexPoint};
    use crate::rendering;
    use crate::rendering::segments::descriptions::TurnDescription;
    use crate::rendering::segments::smooth_segments::segment_cross_sections;
    use crate::snake::SegmentType;

    const CELL_DIM: CellDim = CellDim { side: 50., sin: 43.30127, cos: 25. };

    fn desc(coming_from: Dir, going_to: Dir, turn_fraction: f32) -> SegmentDescription {
        SegmentDescription {
            segment_idx: 0,
            destination: Point { x: 137., y: -91. },
            turn: TurnDescription {
                coming_from,
                going_to,
                fraction: turn_fraction,
            },
            fraction: SegmentFraction { start: 0., end: 1. },
            draw_style: rendering::Style::Smooth,
            segment_type: SegmentType::Normal,
            z_index: 0,
            cell_dim: CELL_DIM,
        }
    }

    /// Every point on the drawn ribbon's edge must sit exactly half a width
    /// from the centerline — that is what makes a distance query against the
    /// centerline a collision test against the graphics.
    #[test]
    fn edges_are_half_a_width_from_the_centerline() {
        let half_width = CELL_DIM.side / 2.;

        for coming_from in Dir::iter() {
            for going_to in Dir::iter() {
                if coming_from == going_to {
                    continue;
                }
                for turn_fraction in [0.05, 0.3, 0.75, 1.] {
                    let desc = desc(coming_from, going_to, turn_fraction);
                    let (cross_sections, _) = segment_cross_sections(&desc);
                    let to_board = desc.board_transform();

                    for (inner, outer, _) in cross_sections {
                        for edge in [inner, outer] {
                            let distance = distance_to_centerline(&desc, desc.fraction, to_board(edge));
                            // a sharp turn's inner edge degenerates to the
                            // pivot, which is the one point the whole width
                            // away from the centerline on the inside
                            assert!(
                                (distance - half_width).abs() < 0.01,
                                "{coming_from:?} -> {going_to:?} at {turn_fraction}: \
                                 edge is {distance} from the centerline, expected {half_width}",
                            );
                        }
                    }
                }
            }
        }
    }

    /// The centerline itself is at distance zero, and the ribbon's midline is
    /// the centerline.
    #[test]
    fn midline_is_the_centerline() {
        for coming_from in Dir::iter() {
            for going_to in Dir::iter() {
                if coming_from == going_to {
                    continue;
                }
                let desc = desc(coming_from, going_to, 1.);
                for step in 0..=10 {
                    let frac = step as f32 / 10.;
                    let point = centerline_at(&desc, frac);
                    let distance = distance_to_centerline(&desc, desc.fraction, point);
                    assert!(
                        distance < 0.01,
                        "{coming_from:?} -> {going_to:?} at {frac}: midline point is {distance} off",
                    );
                }
            }
        }
    }

    fn gtx() -> GameContext {
        use crate::app::game_mode::GameMode;
        use crate::apple::spawn::SpawnPolicy;
        use crate::basic::HexDim;

        GameContext::new(
            HexDim { h: 20, v: 20 },
            CELL_DIM,
            crate::app::Palette::dark(),
            Default::default(),
            SpawnPolicy::None,
            GameMode::Classic,
        )
    }

    /// A snake going straight up, its head `head_fraction` into `head_cell` and
    /// its tail `tail_fraction` out of its trailing cell.
    fn straight_body_at(head_cell: HexPoint, len: usize, head_fraction: f32, tail_fraction: f32) -> Body {
        use crate::basic::Dir;
        use crate::snake::Segment;

        let segments = (0..len)
            .map(|i| Segment {
                segment_type: SegmentType::Normal,
                pos: HexPoint {
                    h: head_cell.h,
                    v: head_cell.v + i as isize,
                },
                coming_from: Dir::D,
                going_to: (i > 0).then_some(Dir::U),
                teleported: None,
                z_index: 0,
            })
            .collect();

        // tail_fraction() is derived from these, so invert it
        let on_board = (len as f32 - 1.) + head_fraction - tail_fraction;

        Body {
            segments,
            dir: Dir::U,
            head_fraction,
            length: on_board,
            emerged: on_board,
            swallowed: 0.,
            length_changes: vec![],
            turn_start: None,
            search_trace: None,
        }
    }

    fn straight_body(len: usize) -> Body {
        straight_body_at(HexPoint { h: 4, v: 4 }, len, 0.5, 0.)
    }

    /// The drawn flesh reaches exactly one cap radius past the shortened end,
    /// which is the whole reason the centerline is shortened at all.
    #[test]
    fn head_base_is_one_cap_radius_behind_the_tip() {
        let gtx = gtx();
        let body = straight_body(5);

        let untruncated = segment_descriptions(&body, &gtx);
        let tip = centerline_at(&untruncated[0], untruncated[0].fraction.end);

        let centerline = Centerline::of(&body, &gtx);
        let base = centerline.head_base().unwrap();

        assert!((centerline.cap_radius - centerline.half_width).abs() < 0.01);
        assert!(
            ((tip - base).magnitude() - centerline.cap_radius).abs() < 0.01,
            "base is {} from the tip, expected {}",
            (tip - base).magnitude(),
            centerline.cap_radius,
        );
    }

    /// The whole point of the exercise: a head entering a cell another snake's
    /// tail is vacating shares that cell with it, but the two are nowhere near
    /// touching. Cell-based collision crashes here; this must not.
    ///
    /// The tail's segment in the contested cell draws nothing — it lies wholly
    /// inside the round tail cap — so this also covers the neighbour fallback.
    #[test]
    fn sharing_a_cell_is_not_touching() {
        let gtx = gtx();
        let contested = HexPoint { h: 4, v: 6 };

        // a head that has just barely entered the contested cell
        let entering = Centerline::of(&straight_body_at(contested, 3, 0.02, 0.), &gtx);
        // a tail that has nearly finished leaving it
        let leaving = Centerline::of(&straight_body_at(HexPoint { h: 4, v: 4 }, 3, 0.5, 0.95), &gtx);

        let probe = entering.head_base().unwrap();
        let threshold = entering.cap_radius + leaving.half_width;
        let distance = leaving.distance_to(2, probe).expect("the tail cap is still there");

        assert!(
            distance > threshold,
            "a head {distance} away from a tail should not crash into it (threshold {threshold})",
        );
    }

    /// The converse: once they really do overlap, they collide.
    #[test]
    fn overlapping_flesh_touches() {
        let gtx = gtx();
        let contested = HexPoint { h: 4, v: 6 };

        let entering = Centerline::of(&straight_body_at(contested, 3, 0.5, 0.), &gtx);
        // this time the tail still fills the contested cell
        let leaving = Centerline::of(&straight_body_at(HexPoint { h: 4, v: 4 }, 3, 0.5, 0.), &gtx);

        let probe = entering.head_base().unwrap();
        let threshold = entering.cap_radius + leaving.half_width;
        let distance = leaving.distance_to(2, probe).unwrap();

        assert!(
            distance <= threshold,
            "overlapping snakes are {distance} apart (threshold {threshold})"
        );
    }

    /// A point beyond the end of a clipped range falls back to the end point,
    /// so clipping shortens the curve rather than making it vanish.
    #[test]
    fn clipping_shortens_the_curve() {
        let mut desc = desc(Dir::D, Dir::U, 1.);
        let tip = centerline_at(&desc, 1.);

        desc.fraction = SegmentFraction { start: 0., end: 0.5 };
        let half = centerline_at(&desc, 0.5);
        let distance = distance_to_centerline(&desc, desc.fraction, tip);

        assert!((distance - (tip - half).magnitude()).abs() < 0.01);
        assert!(distance > 0.01, "the clipped end should no longer reach the tip");
    }
}
