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
//! The round-head cap is intentionally not handled here (to be revisited).

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

/// Arc cross-sections for a turn, or `None` if the turn is too shallow to curve
/// (caller falls back to a straight box).
fn curved_cross_sections(
    description: &SegmentDescription,
    mut turn_fraction: f32,
    fraction: SegmentFraction,
) -> Option<Vec<CrossSection>> {
    // a blunt turn is equivalent to half a sharp turn
    if let TurnType::Blunt(_) = description.turn.turn_type() {
        turn_fraction /= 2.;
    }

    let CellDim { side, sin, cos } = description.cell_dim;

    // distance of the pivot from where it is for a sharp turn
    let pivot_dist = 2. * cos * (1. / turn_fraction - 1.);
    if pivot_dist.is_infinite() {
        // too straight to be drawn as curved
        return None;
    }
    let pivot = Point { x: side + cos + pivot_dist, y: 0. };

    // A complete sharp turn has inner_radius == 0 (the inner edge is the pivot
    // point). That is fine here: the ribbon reuses the pivot as the inner point
    // of every cross-section, each with its own frac.
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
            match curved_cross_sections(description, description.turn.fraction, description.fraction) {
                Some(sections) => (sections, cw),
                // fell back to a symmetric box: no flip needed
                None => (straight_cross_sections(description.cell_dim, description.fraction), false),
            }
        }
    }
}
