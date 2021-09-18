use std::{cmp::max, f32::consts::TAU};

use crate::{
    app::rendering::segments::{
        descriptions::{SegmentDescription, SegmentFraction, TurnType},
        point_factory::SegmentRenderer,
    },
    basic::{CellDim, Point},
};

pub struct SmoothSegments;

/// Number of segments for a full turn, a lower number results
/// in a lower resolution, i.e. jagged edges
const NUM_ANGLE_SEGMENTS: usize = 10;

/// Return the upper intersection point between two circles
///  - p0, p1 are the centers of the circles
///  - r0, r1 are the radii
///  - p3, p4 are the intersection points
///  - p2 is the intersection between the lines p0-p1 and p3-p4
///  - d is the distance p0-p1
///  - a is the distance p0-p2
///  - h is the distance p2-p3 (equal to p2-p4)
fn upper_intersection_point(p0: Point, r0: f32, p1: Point, r1: f32) -> Point {
    let d: f32 = (p0 - p1).magnitude();
    let a = (r0.powi(2) - r1.powi(2) + d.powi(2)) / (2. * d);
    let p2: Point = p0 + (a / d) * (p1 - p0);
    let h = (r0.powi(2) - a.powi(2)).sqrt();
    let p3 = Point {
        x: p2.x - (h / d) * (p1.y - p0.y),
        y: p2.y + (h / d) * (p1.x - p0.x),
    };
    p3
}

impl SegmentRenderer for SmoothSegments {
    fn render_default_straight_segment(description: &SegmentDescription) -> Vec<Point> {
        let SegmentDescription { cell_dim, fraction, .. } = description;

        let CellDim { side, sin, cos } = *cell_dim;
        let SegmentFraction { start, end } = *fraction;

        // top-left, top-right, bottom-right, bottom-left
        vec![
            Point { x: cos, y: end * 2. * sin },
            Point { x: cos + side, y: end * 2. * sin },
            Point { x: cos + side, y: start * 2. * sin },
            Point { x: cos, y: start * 2. * sin },
        ]
    }

    fn render_default_curved_segment(
        description: &SegmentDescription,
        mut turn: f32,
    ) -> Vec<Point> {
        // a blunt turn is equivalent to half a sharp turn
        if let TurnType::Blunt(_) = description.turn.turn_type() {
            turn /= 2.;
        }

        let SegmentDescription { cell_dim, fraction, .. } = description;

        let CellDim { side, sin, cos } = *cell_dim;

        // distance of the pivot from where it is for a sharp turn
        let pivot_dist = 2. * cos * (1. / turn - 1.);
        // too straight to be drawn as curved, default to straight drawing
        if pivot_dist.is_infinite() {
            return Self::render_default_straight_segment(description);
        }
        let pivot = Point { x: side + cos + pivot_dist, y: 0. };

        // We imagine a circle in which the cell's hexagon is inscribed,
        // we also imagine a circle around the pivot tracing the outer path,
        // we find the intersection between these circles to determine the
        // angle (amount of path) to actually trace.
        // The angle is the same for both inner and outer circles, the
        // calculation would be equivalent for the inner circle.
        let total_angle = if (turn - 1.).abs() < f32::EPSILON {
            // shortcut for a complete turn
            TAU / 3.
        } else {
            // find the (upper) intersection point between the two circles
            let p0 = Point { x: cos + side / 2., y: sin };
            let r0 = ((side / 2.).powi(2) + sin.powi(2)).sqrt();
            let r1 = side + pivot_dist;
            let intersection_point = upper_intersection_point(p0, r0, pivot, r1);

            // find the angle around the pivot for when the pivot is right or left
            // of the intersection point
            if intersection_point.x <= pivot.x {
                (intersection_point.y / (pivot.x - intersection_point.x)).atan()
            } else {
                TAU / 2. - (intersection_point.y / (intersection_point.x - pivot.x)).atan()
            }
        };

        let inner_line_start = Point { x: cos + side, y: 0. };
        let outer_line_start = Point { x: cos, y: 0. };

        let fraction_size = fraction.end - fraction.start;
        let num_angle_segments = NUM_ANGLE_SEGMENTS as f32 * fraction_size;
        let num_angle_segments = max(1, num_angle_segments as usize);

        let mut points = Vec::with_capacity(num_angle_segments * 2 + 2);
        let start_angle = fraction.start * total_angle;
        let end_angle = fraction.end * total_angle;
        let angle_diff = end_angle - start_angle;

        let inner_line = (0..=num_angle_segments).map(move |i| {
            let i = i as f32 / num_angle_segments as f32;
            let angle = start_angle + i * angle_diff;
            inner_line_start.rotate_counterclockwise(pivot, angle)
        });

        // outer line in opposite direction (notice call to rev)
        let outer_line = (0..=num_angle_segments).rev().map(move |i| {
            let i = i as f32 / num_angle_segments as f32;
            let angle = start_angle + i * angle_diff;
            outer_line_start.rotate_counterclockwise(pivot, angle)
        });

        points.extend(inner_line);
        points.extend(outer_line);

        // TODO: if this appears again, just forgo drawing the segment
        if points.len() < 3 {
            assert_eq!(points.len(), 2);
            panic!("segment with only 2 points");
        }

        points
    }
}
