use ggez::mint;
use std::f32::consts::{PI, TAU};
use std::iter;

use crate::basic::{CellDim, Point};
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnType};
use crate::rendering::segments::point_factory::SegmentRenderer;
use itertools::Itertools;
use lyon_geom::euclid::default::Point2D;
use lyon_geom::{Angle, Arc};

pub struct SmoothSegments;

// TODO: make this a variable parameter based on zoom level
const TOLERANCE: f32 = 0.5;

// TODO: this documentation is confusing and probably wrong
// (wouldn't a = r0 !?) (it isn't)
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
    fn render_default_straight_segment(
        description: &SegmentDescription,
        fraction: SegmentFraction,
        next_fraction: Option<SegmentFraction>,
        previous_fraction: Option<SegmentFraction>,
    ) -> Vec<Point> {
        let CellDim { side, sin, cos } = description.cell_dim;

        // TODO: assert this upstream and figure out how to handle snake growing from 0
        // assert!(
        //     fraction.end - fraction.start >= side,
        //     "segment too short, must be at least as long as it is wide"
        // );

        let head_radius = side / 2.;

        // the length of a full straight segment (height of a hexagon)
        let length = 2. * sin;

        dbg!(next_fraction);
        dbg!(fraction);
        dbg!(previous_fraction);

        if let Some(next_fraction) = next_fraction {
            // the segment has already entered the next cell

            if next_fraction.end < head_radius {
                println!("A");
                // part of the head curvature is still visible in this cell
                // (as two separate arcs)
            } else {
                println!("B");
                // no head curvature is visible in this cell
            }
        } else {
            // the segment ends in this cell

            if fraction.end < head_radius {
                println!("C");
                // only part of the head curvature is visible in this cell
            } else {
                println!("D");
                // the whole head curvature is visible in this cell
                // println!("hit");

                let center = Point {
                    x: cos + 0.5 * side,
                    // fraction.end >= head_radius
                    y: fraction.end - head_radius,
                }
                .into();

                let head = Arc {
                    center,
                    radii: Point::square(head_radius).into(),
                    start_angle: Angle { radians: 0. },
                    sweep_angle: Angle { radians: PI },
                    x_rotation: Angle { radians: 0. },
                };

                return head.flattened(TOLERANCE).map(From::from).collect();
            }
        }

        // top-left, top-right, bottom-right, bottom-left
        vec![
            Point { x: cos, y: fraction.end * length },
            Point {
                x: cos + side,
                y: fraction.end * length,
            },
            Point {
                x: cos + side,
                y: fraction.start * length,
            },
            Point { x: cos, y: fraction.start * length },
        ]
    }

    fn render_default_curved_segment(
        description: &SegmentDescription,
        mut turn_fraction: f32,
        fraction: SegmentFraction,
    ) -> Vec<Point> {
        // a blunt turn is equivalent to half a sharp turn
        if let TurnType::Blunt(_) = description.turn.turn_type() {
            turn_fraction /= 2.;
        }

        let CellDim { side, sin, cos } = description.cell_dim;

        let pivot = {
            // distance of the pivot from where it is for a sharp turn
            let pivot_dist = 2. * cos * (1. / turn_fraction - 1.);
            // too straight to be drawn as curved, default to straight drawing
            if pivot_dist.is_infinite() {
                panic!("add previous and next parameters to curved segment");
                // return Self::render_default_straight_segment(description, fraction);
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
