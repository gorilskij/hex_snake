use std::f32::consts::TAU;
use std::iter;

use crate::basic::{CellDim, Point};
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnType};
use crate::rendering::segments::point_factory::SegmentRenderer;
use itertools::Itertools;
use lyon_geom::{Angle, Arc};

pub struct SmoothSegments;

// TODO: make this a variable parameter based on zoom level
const TOLERANCE: f32 = 0.5;

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
    // p3
    Point {
        x: p2.x - (h / d) * (p1.y - p0.y),
        y: p2.y + (h / d) * (p1.x - p0.x),
    }
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
        mut turn_fraction: f32,
    ) -> Vec<Point> {
        // a blunt turn is equivalent to half a sharp turn
        if let TurnType::Blunt(_) = description.turn.turn_type() {
            turn_fraction /= 2.;
        }

        let SegmentDescription { cell_dim, fraction, .. } = description;

        let CellDim { side, sin, cos } = *cell_dim;

        let pivot = {
            // distance of the pivot from where it is for a sharp turn
            let pivot_dist = 2. * cos * (1. / turn_fraction - 1.);
            // too straight to be drawn as curved, default to straight drawing
            if pivot_dist.is_infinite() {
                return Self::render_default_straight_segment(description);
            }
            Point { x: side + cos + pivot_dist, y: 0. }
        };

        let inner_radius = pivot.x - side - cos;
        let outer_radius = pivot.x - cos;

        // We imagine a circle in which the cell's hexagon is inscribed,
        // we also imagine a circle around the pivot tracing the outer path,
        // we find the intersection between these circles to determine the
        // angle (amount of path) to actually trace.
        // The angle is the same for both inner and outer circles, the
        // calculation would be equivalent for the inner circle.
        let total_angle = if (turn_fraction - 1.).abs() < f32::EPSILON {
            // shortcut for a complete turn
            TAU / 3.
        } else {
            // find the (upper) intersection point between the two circles
            let p0 = Point { x: cos + side / 2., y: sin };
            let r0 = ((side / 2.).powi(2) + sin.powi(2)).sqrt();
            let intersection_point = upper_intersection_point(p0, r0, pivot, outer_radius);

            // find the angle around the pivot for when the pivot is right or left
            // of the intersection point
            if intersection_point.x <= pivot.x {
                (intersection_point.y / (pivot.x - intersection_point.x)).atan()
            } else {
                TAU / 2. - (intersection_point.y / (intersection_point.x - pivot.x)).atan()
            }
        };

        let start_radians = fraction.start * total_angle;
        let end_radians = fraction.end * total_angle;

        let center = pivot.into();
        let start_angle = Angle { radians: TAU / 2. - start_radians };
        let sweep_angle = Angle {
            radians: start_radians - end_radians,
        };
        let x_rotation = Angle { radians: 0. };

        let inner_arc = Arc {
            center,
            radii: Point::square(inner_radius).into(),
            start_angle,
            sweep_angle,
            x_rotation,
        };

        let outer_arc = Arc {
            center,
            radii: Point::square(outer_radius).into(),
            start_angle: inner_arc.end_angle(),
            sweep_angle: -sweep_angle,
            x_rotation,
        };

        let points = iter::once(inner_arc.sample(0.))
            .chain(inner_arc.flattened(TOLERANCE))
            .chain(iter::once(outer_arc.sample(0.)))
            .chain(outer_arc.flattened(TOLERANCE))
            .map(Point::from)
            .collect_vec();

        // TODO: if this appears again, just forgo drawing the segment
        if points.len() < 3 {
            assert_eq!(points.len(), 2);
            panic!("segment with only 2 points");
        }

        points
    }
}
