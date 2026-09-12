//! Passability marks: a segment its snake can pass through (see
//! [`EatMechanics::is_marked`]) is marked in a darker shade of its own color —
//! two round-ended lines along its edges in the smooth style, a small hexagon
//! in its middle in the hexagon style — so it can be told apart from
//! look-alikes the snake would crash into, like other snakes' eaten segments.
//!
//! Where a neighboring segment is marked too, the lines run flat up to the
//! shared edge, so a run of marked segments shows one continuous pair of lines.
//!
//! [`EatMechanics::is_marked`]: crate::snake::eat_mechanics::EatMechanics::is_marked

use std::f32::consts::FRAC_PI_2;

use crate::basic::{CellDim, Point};
use crate::rendering;
use crate::rendering::segments::descriptions::{SegmentDescription, TurnType};
use crate::rendering::segments::smooth_segments::{arc_params, cross_section_at, ArcParams};
use crate::rendering::shape::{Hexagon, Shape};
use crate::support::mesh::{build_shaded_polygon, build_shaded_ribbon, Mesh};

// Dimensions are first guesses, to be tuned by eye.

/// Brightness of the marks relative to the segment's own color.
const BRIGHTNESS: f32 = 0.8;

/// Gap between each line and its edge of the body, as a fraction of the body's
/// width.
const LINE_INSET: f32 = 0.12;

/// Width of each line, as a fraction of the body's width.
const LINE_WIDTH: f32 = 0.1;

/// Segment fractions of the centers of a line's round ends (the rounding
/// sticks out by half the line's width beyond these).
const LINE_START: f32 = 0.25;
const LINE_END: f32 = 0.75;

/// Target on-screen spacing (px) between successive cross-sections of a
/// curved line.
const LINE_STEP: f32 = 3.0;

/// Cross-sections per round end.
const ROUND_STEPS: usize = 6;

/// Size of the hexagon-style mark relative to the cell.
const HEXAGON_SCALE: f32 = 0.4;

/// Which ends of a segment's lines continue into a neighboring segment's lines
/// (running flat to the shared edge instead of ending round).
#[derive(Copy, Clone, Debug)]
pub struct Joins {
    /// Towards the tail: the older neighbor is marked.
    pub tail: bool,
    /// Towards the head: the younger neighbor is marked (for the head segment,
    /// the next head segment is already known to be marked).
    pub head: bool,
}

/// Build a segment's marks (whether it is marked is the caller's business).
pub fn build_marks(desc: &SegmentDescription, joins: Joins, num_segments: usize, lut_size: usize) -> Mesh {
    match desc.draw_style {
        rendering::Style::Smooth => build_lines(desc, joins, num_segments, lut_size),
        rendering::Style::Hexagon => build_hexagon(desc, num_segments),
    }
}

/// The two lines following the segment's edges. On a turn they are arcs
/// around the pivot, so the inner one is naturally shorter; a sharp turn's
/// inner edge is a single point, so its inner line shrinks to a dot.
fn build_lines(desc: &SegmentDescription, joins: Joins, num_segments: usize, lut_size: usize) -> Mesh {
    let arc = arc_params(desc);
    let sharp = arc.is_some() && matches!(desc.turn.turn_type(), TurnType::Sharp(_));
    let inner_centers = if sharp { (0.5, 0.5) } else { (LINE_START, LINE_END) };

    let line = |across, centers| build_line(desc, arc.as_ref(), across, centers, joins, num_segments, lut_size);
    Mesh::combine([
        line((LINE_INSET, LINE_INSET + LINE_WIDTH), inner_centers),
        line((1. - LINE_INSET - LINE_WIDTH, 1. - LINE_INSET), (LINE_START, LINE_END)),
    ])
}

/// One line, spanning `across` of the body's width (0 is the inner edge, 1 the
/// outer) and running between the round-end `centers` — or flat to the edge on
/// a joined end. It is cut to the segment's drawn fraction, so it never pokes
/// out of the snake's round caps.
fn build_line(
    desc: &SegmentDescription,
    arc: Option<&ArcParams>,
    across: (f32, f32),
    centers: (f32, f32),
    joins: Joins,
    num_segments: usize,
    lut_size: usize,
) -> Mesh {
    let mid = (across.0 + across.1) / 2.;
    let half_width = (across.1 - across.0) / 2.;

    // the line's own path length per unit of segment fraction (an arc's
    // length depends on its radius), to keep its round ends circular
    let path_len = match arc {
        Some(arc) => (arc.inner_radius + mid * (arc.outer_radius - arc.inner_radius)) * arc.total_angle,
        None => desc.cell_dim.height(),
    };
    let radius = half_width * desc.cell_dim.side / path_len;

    let (a, b) = (
        if joins.tail { 0. } else { centers.0 },
        if joins.head { 1. } else { centers.1 },
    );
    let lo = desc.fraction.start.max(if joins.tail { 0. } else { a - radius });
    let hi = desc.fraction.end.min(if joins.head { 1. } else { b + radius });
    if hi <= lo {
        return Mesh::empty();
    }

    // relative width at a fraction: 1 along the line, a circular profile
    // over a round end
    let width_at = |frac: f32| {
        let past_end = if !joins.tail && frac < a {
            (a - frac) / radius
        } else if !joins.head && frac > b {
            (frac - b) / radius
        } else {
            0.
        };
        (1. - past_end.powi(2)).max(0.).sqrt()
    };

    // sample densely over the round ends (evenly in angle), along the length
    // only as needed to follow a curve, and exactly at the cuts
    let round_offsets = || (0..=ROUND_STEPS).map(|k| radius * (k as f32 / ROUND_STEPS as f32 * FRAC_PI_2).cos());
    let body_steps = match arc {
        Some(_) => ((path_len * (b - a) / LINE_STEP).ceil() as usize).clamp(1, 32),
        None => 1,
    };
    let mut fracs: Vec<f32> = [lo, hi]
        .into_iter()
        .chain(round_offsets().map(|offset| a - offset))
        .chain(round_offsets().map(|offset| b + offset))
        .chain((0..=body_steps).map(|k| a + (b - a) * k as f32 / body_steps as f32))
        .filter(|frac| (lo..=hi).contains(frac))
        .collect();
    fracs.sort_by(f32::total_cmp);
    fracs.dedup_by(|x, y| (*x - *y).abs() < 1e-5);

    let sections: Vec<(Point, Point, f32)> = fracs
        .into_iter()
        .map(|frac| {
            let (inner, outer) = cross_section_at(desc, frac);
            let at = |t: f32| inner + (outer - inner) * t;
            let half = half_width * width_at(frac);
            (at(mid - half), at(mid + half), frac)
        })
        .collect();

    build_shaded_ribbon(
        &sections,
        desc.seg_bounds(num_segments, lut_size),
        BRIGHTNESS,
        desc.u_of(num_segments),
        desc.board_transform(),
    )
}

/// A small flat hexagon in the middle of the cell.
fn build_hexagon(desc: &SegmentDescription, num_segments: usize) -> Mesh {
    let mark_dim = CellDim::from(desc.cell_dim.side * HEXAGON_SCALE);
    let offset = desc.destination + Hexagon::center(desc.cell_dim) - Hexagon::center(mark_dim);
    let points: Vec<Point> = Hexagon::new(mark_dim).translate(offset).into();
    // one flat color, like the segment itself: sample its midpoint
    let u = desc.u_of(num_segments)(0.5);
    build_shaded_polygon(&points, BRIGHTNESS, move |_| (u, 0.5), |p| p)
}
