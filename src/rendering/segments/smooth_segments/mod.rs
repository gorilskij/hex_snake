//! Cross-section ribbons for snake segments (smooth draw style).
//!
//! Each segment is a triangle strip of cross-sections marched along the body: a
//! straight box is two cross-sections, a turn is an arc of them. Every
//! cross-section is `(inner, outer, frac)` — the two edge points plus the
//! segment-local body fraction (0 tail-side, 1 head-side). Color comes from the
//! shader sampling the palette LUT by `frac`; because adjacent cross-sections
//! share vertices, every radial edge is a single constant-`frac` line, so there
//! are no seams. A complete sharp turn has `inner_radius == 0`: the inner point
//! is simply the pivot, reused across cross-sections with different `frac` — a
//! true single point, no hole, no seam.
//!
//! Round end caps (see `cap.rs`) reuse this module's geometry: a cap is just
//! more cross-sections with shrinking width, sampled via [`cross_section_at`].

use std::f32::consts::{PI, TAU};

use crate::basic::{CellDim, Point};
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnDirection, TurnType};

/// One cross-section of a segment ribbon, in default orientation.
/// `(inner_edge_point, outer_edge_point, body_fraction)`.
pub type CrossSection = (Point, Point, f32);

/// Target on-screen spacing (px) between successive cross-sections of a turn.
const ARC_STEP: f32 = 3.0;

/// Return the upper intersection point between two circles (used to find how far
/// around the pivot a turn's arc must sweep).
fn upper_intersection_point(p0: Point, r0: f32, p1: Point, r1: f32) -> Point {
    let d: f32 = (p0 - p1).magnitude();
    let a = (r0.powi(2) - r1.powi(2) + d.powi(2)) / (2. * d);
    let p2: Point = p0 + (a / d) * (p1 - p0);
    let h = (r0.powi(2) - a.powi(2)).sqrt();
    Point {
        x: p2.x - (h / d) * (p1.y - p0.y),
        y: p2.y + (h / d) * (p1.x - p0.x),
    }
}

/// Straight box as two cross-sections: x across `[cos, cos+side]`, y along
/// `[start*height, end*height]`.
fn straight_cross_sections(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<CrossSection> {
    let CellDim { side, cos, .. } = cell_dim;
    let height = cell_dim.height();
    vec![
        (
            Point { x: cos, y: fraction.start * height },
            Point { x: cos + side, y: fraction.start * height },
            fraction.start,
        ),
        (
            Point { x: cos, y: fraction.end * height },
            Point { x: cos + side, y: fraction.end * height },
            fraction.end,
        ),
    ]
}

/// The geometry of a turn's arc in default orientation (counterclockwise; the
/// clockwise mirror is applied by the board transform).
pub struct ArcParams {
    pub pivot: Point,
    /// A complete sharp turn has `inner_radius == 0` (the inner edge is the
    /// pivot point).
    pub inner_radius: f32,
    pub outer_radius: f32,
    /// Angle swept around the pivot from fraction 0 to fraction 1.
    pub total_angle: f32,
}

/// Compute the arc parameters of a segment's turn, or `None` if the segment is
/// straight or the turn is too shallow to curve (drawn as a straight box).
pub fn arc_params(description: &SegmentDescription) -> Option<ArcParams> {
    let mut turn_fraction = description.turn.fraction;
    match description.turn.turn_type() {
        TurnType::Straight => return None,
        // a blunt turn is equivalent to half a sharp turn
        TurnType::Blunt(_) => turn_fraction /= 2.,
        TurnType::Sharp(_) => {}
    }

    let CellDim { side, sin, cos } = description.cell_dim;

    // distance of the pivot from where it is for a sharp turn
    let pivot_dist = 2. * cos * (1. / turn_fraction - 1.);
    if pivot_dist.is_infinite() {
        // too straight to be drawn as curved
        return None;
    }
    let pivot = Point { x: side + cos + pivot_dist, y: 0. };

    let inner_radius = pivot.x - side - cos;
    let outer_radius = pivot.x - cos;

    // Angle (amount of path) swept around the pivot for a full turn.
    let total_angle = if (turn_fraction - 1.).abs() < f32::EPSILON {
        TAU / 3.
    } else {
        let p0 = Point { x: cos + side / 2., y: sin };
        let r0 = ((side / 2.).powi(2) + sin.powi(2)).sqrt();
        let intersection_point = upper_intersection_point(p0, r0, pivot, outer_radius);
        if intersection_point.x <= pivot.x {
            (intersection_point.y / (pivot.x - intersection_point.x)).atan()
        } else {
            TAU / 2. - (intersection_point.y / (intersection_point.x - pivot.x)).atan()
        }
    };

    Some(ArcParams {
        pivot,
        inner_radius,
        outer_radius,
        total_angle,
    })
}

/// Path length of a full (fraction 0 to 1) traversal of this segment's cell,
/// measured along the body's centerline.
pub fn full_path_length(description: &SegmentDescription) -> f32 {
    match arc_params(description) {
        Some(arc) => (arc.inner_radius + arc.outer_radius) / 2. * arc.total_angle,
        None => description.cell_dim.height(),
    }
}

/// The `(inner, outer)` cross-section line at a given fraction of this segment,
/// in default orientation (same space as [`segment_cross_sections`]).
pub fn cross_section_at(description: &SegmentDescription, frac: f32) -> (Point, Point) {
    match arc_params(description) {
        Some(arc) => {
            // angle around the pivot is PI at fraction 0, decreasing to
            // PI - total_angle at fraction 1
            let theta = PI - frac * arc.total_angle;
            let (s, c) = theta.sin_cos();
            (
                Point {
                    x: arc.pivot.x + arc.inner_radius * c,
                    y: arc.inner_radius * s,
                },
                Point {
                    x: arc.pivot.x + arc.outer_radius * c,
                    y: arc.outer_radius * s,
                },
            )
        }
        None => {
            let CellDim { side, cos, .. } = description.cell_dim;
            let y = frac * description.cell_dim.height();
            (Point { x: cos, y }, Point { x: cos + side, y })
        }
    }
}

/// Arc cross-sections for a turn, or `None` if the turn is too shallow to curve
/// (caller falls back to a straight box).
fn curved_cross_sections(description: &SegmentDescription, fraction: SegmentFraction) -> Option<Vec<CrossSection>> {
    let ArcParams {
        pivot,
        inner_radius,
        outer_radius,
        total_angle,
    } = arc_params(description)?;

    // Sample the arc from fraction.start to fraction.end. Angle around the pivot
    // is PI at fraction 0, decreasing to PI - total_angle at fraction 1.
    let start_angle = PI - fraction.start * total_angle;
    let end_angle = PI - fraction.end * total_angle;

    let arc_len = outer_radius * (fraction.end - fraction.start).abs() * total_angle;
    let steps = ((arc_len / ARC_STEP).ceil() as usize).clamp(4, 64);

    let sections = (0..=steps)
        .map(|k| {
            let t = k as f32 / steps as f32;
            let theta = start_angle + (end_angle - start_angle) * t;
            let (s, c) = theta.sin_cos();
            let inner = Point { x: pivot.x + inner_radius * c, y: inner_radius * s };
            let outer = Point { x: pivot.x + outer_radius * c, y: outer_radius * s };
            let frac = fraction.start + (fraction.end - fraction.start) * t;
            (inner, outer, frac)
        })
        .collect();

    Some(sections)
}

/// Build a smooth-style segment as a ribbon of cross-sections in default
/// orientation, plus whether it must be flipped horizontally (clockwise turns)
/// when placed on the board.
pub fn segment_cross_sections(description: &SegmentDescription) -> (Vec<CrossSection>, bool) {
    match description.turn.turn_type() {
        TurnType::Straight => (straight_cross_sections(description.cell_dim, description.fraction), false),
        TurnType::Blunt(dir) | TurnType::Sharp(dir) => {
            let cw = dir == TurnDirection::Clockwise;
            match curved_cross_sections(description, description.fraction) {
                Some(sections) => (sections, cw),
                // fell back to a symmetric box: no flip needed
                None => (straight_cross_sections(description.cell_dim, description.fraction), false),
            }
        }
    }
}
