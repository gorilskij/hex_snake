use std::f32::consts::{PI, TAU};
use std::io::Read;
use std::iter;

use crate::basic::{CellDim, Point};
use crate::rendering::segments::descriptions::{
    RoundHeadDescription, SegmentDescription, SegmentFraction, TurnType,
};
use crate::rendering::segments::point_factory::SegmentRenderer;
use itertools::{chain, Itertools};
use lyon_geom::{Angle, Arc, SvgArc};
pub struct SmoothSegments;

// TODO: make this a variable parameter based on zoom level
const TOLERANCE: f32 = 0.05;

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

fn extreme_points(center: Point, radius: f32, start_angle: f32, end_angle: f32) -> (Point, Point) {
    let starting_point = center + Point { x: radius, y: 0.0 };
    let first_point = starting_point.rotate_clockwise(center, start_angle);
    let last_point = starting_point.rotate_clockwise(center, end_angle);
    (first_point, last_point)
}

impl SmoothSegments {
    fn render_arc_tip(description: &SegmentDescription, fraction: SegmentFraction) -> Vec<Point> {
        let CellDim { side, sin, cos } = description.cell_dim;
        let head_radius = side / 2.;

        let center = Point {
            x: cos + 0.5 * side,
            // fraction.end >= head_radius
            y: fraction.end * 2. * sin - head_radius,
        };

        let slice_thickness = (fraction.end - fraction.start) * 2. * side;
        let start_angle = ((head_radius - slice_thickness) / head_radius).asin();
        let end_angle = PI - start_angle;

        // TODO: custom arc implementation? seriously? or find a better arc
        let arc = Arc {
            center: center.into(),
            radii: Point::square(head_radius).into(),
            start_angle: Angle { radians: start_angle },
            sweep_angle: Angle { radians: end_angle - start_angle },
            x_rotation: Angle { radians: 0. },
        };

        let (first_point, last_point) = extreme_points(center, head_radius, start_angle, end_angle);

        let points: Vec<Point> = iter::once(first_point)
            .chain(arc.flattened(TOLERANCE).map(From::from))
            .chain(iter::once(last_point))
            .collect();

        points
    }

    fn render_split_arc(
        description: &SegmentDescription,
        fraction: SegmentFraction,
        // The base of the round head (center of the circle, y coordinate), even if negative
        head_base: f32,
    ) -> Vec<Point> {
        let CellDim { side, sin, cos } = description.cell_dim;
        let head_radius = side / 2.;

        let center = Point {
            x: cos + 0.5 * side,
            // fraction.end >= head_radius
            y: head_base,
        };

        let d1 = fraction.end * 2. * sin - head_base;
        let d2 = fraction.start * 2. * sin - head_base;

        let start_angle1 = (d1 / head_radius).atan();
        let end_angle1 = (d2 / head_radius).atan();
        // start 2 mirrors end 1, end 2 mirrors start 1
        let start_angle2 = PI - end_angle1;
        let end_angle2 = PI - start_angle1;

        let arc1 = Arc {
            center: center.into(),
            radii: Point::square(head_radius).into(),
            start_angle: Angle { radians: start_angle1 },
            sweep_angle: Angle { radians: end_angle1 - start_angle1 },
            x_rotation: Angle { radians: 0. },
        };

        let arc2 = Arc {
            center: center.into(),
            radii: Point::square(head_radius).into(),
            start_angle: Angle { radians: start_angle2 },
            sweep_angle: Angle { radians: end_angle2 - start_angle2 },
            x_rotation: Angle { radians: 0. },
        };

        let (first1, last1) = extreme_points(center, head_radius, start_angle1, end_angle1);
        let (first2, last2) = extreme_points(center, head_radius, start_angle2, end_angle2);

        chain!(
            iter::once(first1),
            arc1.flattened(TOLERANCE).map(Into::into),
            iter::once(last1),
            iter::once(first2),
            arc2.flattened(TOLERANCE).map(Into::into),
            iter::once(last2),
        )
        .collect()
    }
}

impl SegmentRenderer for SmoothSegments {
    fn render_default_straight_segment(
        description: &SegmentDescription,
        fraction: SegmentFraction,
        round_head: RoundHeadDescription,
    ) -> Vec<Point> {
        let CellDim { side, sin, cos } = description.cell_dim;
        let head_radius = side / 2.;

        // TODO: assert this upstream and figure out how to handle snake growing from 0
        // assert!(
        //     fraction.end - fraction.start >= side,
        //     "segment too short, must be at least as long as it is wide"
        // );

        // the length of a full straight segment (height of a hexagon)
        let length = 2. * sin;

        use RoundHeadDescription::*;

        // TODO: actually pass a boolean flag for first and last subsegments in the snake
        match round_head {
            Tip { segment_end } => {
                println!("TIP");
                if fraction.end == segment_end {
                    println!("tip A: {}-{}", fraction.start, fraction.end);
                    return Self::render_arc_tip(description, fraction);
                } else {
                    println!("tip B: {}-{}", fraction.start, fraction.end);
                    let head_base = segment_end * 2. * sin - head_radius;
                    assert!(head_base <= 0.);
                    return Self::render_split_arc(description, fraction, head_base);
                }
            }
            Full { segment_end } => {
                // println!("FULL");
                if fraction.end == segment_end {
                    return Self::render_arc_tip(description, fraction);
                } else {
                    let head_base = segment_end * 2. * sin - head_radius;
                    if fraction.start * 2. * sin >= head_base {
                        assert!(head_base >= 0.);
                        return Self::render_split_arc(description, fraction, head_base);
                    } else {
                        // println!("NO DRAW")
                    }
                }
            }
            Tail { prev_segment_end } => {
                // println!("TAIL");
                let head_base = 2. * sin + prev_segment_end - head_radius;
                assert!(dbg!(head_base) >= sin);
                assert!(head_base <= 2. * sin);
                if fraction.start * 2. * sin >= head_base {
                    return Self::render_split_arc(description, fraction, head_base);
                }
            }
            Gone => {
                // println!("GONE");
            }
        }
        // return vec![];

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
        round_head: RoundHeadDescription,
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
