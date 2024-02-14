use std::f32::consts::{PI, TAU};
use std::iter;

use itertools::Itertools;
use lyon_geom::{Angle, Arc};

use crate::basic::{CellDim, Dir, Point};
use crate::rendering::clean_arc::CleanArc;
use crate::rendering::segments::descriptions::{
    Polygon, RoundHeadDescription, SegmentDescription, SegmentFraction, TurnDirection, TurnType,
};
use crate::rendering::segments::point_factory::SegmentRenderer;
use crate::rendering::shape::ShapePoints;

mod subsegments;

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

fn render_arc_tip(fraction: SegmentFraction, center: Point, angle: f32, cell_dim: CellDim) -> Vec<Point> {
    let head_radius = cell_dim.side / 2.;
    let slice_thickness = (fraction.end - fraction.start) * 2. * cell_dim.side;

    let start_angle = ((head_radius - slice_thickness) / head_radius).asin();
    let end_angle = PI - start_angle;

    CleanArc {
        center,
        radius: head_radius,
        start_angle: start_angle + angle,
        end_angle: end_angle + angle,
    }
    .flattened(TOLERANCE)
    .collect()
}

fn render_arc_tip_straight(description: &SegmentDescription, fraction: SegmentFraction) -> Vec<Point> {
    let CellDim { side, sin, cos } = description.cell_dim;
    let head_radius = side / 2.;

    let center = Point {
        x: cos + 0.5 * side,
        y: fraction.end * 2. * sin - head_radius,
    };

    render_arc_tip(fraction, center, 0., description.cell_dim)

    // let slice_thickness = (fraction.end - fraction.start) * 2. * side;
    //
    // let start_angle = ((head_radius - slice_thickness) / head_radius).asin();
    // let end_angle = PI - start_angle;
    //
    // CleanArc {
    //     center,
    //     radius: head_radius,
    //     start_angle,
    //     end_angle,
    // }
    // .flattened(TOLERANCE)
    // .collect()
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

    let arc1 = CleanArc {
        center,
        radius: head_radius,
        start_angle: (d1 / head_radius).asin(),
        end_angle: (d2 / head_radius).asin(),
    };

    let arc2 = CleanArc {
        center,
        radius: head_radius,
        // start 2 mirrors end 1, end 2 mirrors start 1
        start_angle: PI - arc1.end_angle,
        end_angle: PI - arc1.start_angle,
    };

    arc2.flattened(TOLERANCE).chain(arc1.flattened(TOLERANCE)).collect()
}

fn render_box(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
    let CellDim { side, cos, .. } = cell_dim;
    let height = cell_dim.height();
    vec![
        Point { x: cos, y: fraction.end * height },
        Point {
            x: cos + side,
            y: fraction.end * height,
        },
        Point {
            x: cos + side,
            y: fraction.start * height,
        },
        Point { x: cos, y: fraction.start * height },
    ]
}

enum PartOfRoundHead {
    Not,
    Partly,
    Fully,
}

fn render_default_straight_segment(
    description: &SegmentDescription,
    subsegment_idx: usize,
    fraction: SegmentFraction,
    round_head: RoundHeadDescription,
) -> Vec<Point> {
    let CellDim { side, cos, .. } = description.cell_dim;
    let head_radius = side / 2.;

    // TODO: assert this upstream and figure out how to handle snake growing from 0
    // assert!(
    //     fraction.end - fraction.start >= side,
    //     "segment too short, must be at least as long as it is wide"
    // );

    let height = description.cell_dim.height();
    let subsegment_start_y = fraction.start * height;
    let subsegment_end_y = fraction.end * height;
    // the tip of the snake (could be out of bounds for the current cell)
    use RoundHeadDescription::*;
    let tip_y = match round_head {
        Tip { segment_end } => segment_end * height,
        Full { segment_end } => segment_end * height,
        Tail { prev_segment_end } => (1. + prev_segment_end) * height,
        Gone => f32::MAX,
    };

    use PartOfRoundHead::*;
    let part_of_round_head = if tip_y - subsegment_start_y <= head_radius {
        Fully
    } else if tip_y - subsegment_end_y <= head_radius {
        Partly
    } else {
        Not
    };

    if description.segment_idx == 0 && subsegment_idx == 0 {
        match part_of_round_head {
            Fully => render_arc_tip_straight(description, fraction),
            Partly => todo!(),
            Not => unreachable!("the first segment of the snake should always be part of the round head"),
        }
    } else {
        let head_base = tip_y - head_radius;
        match part_of_round_head {
            Fully => render_split_arc(description, fraction, head_base),
            Partly => {
                // the fraction at which the segment starts being part of the round head
                let head_base_start = head_base / height;
                let round_fraction = SegmentFraction { start: head_base_start, ..fraction };

                // TODO: figure out maximum number of points and use stack vectors instead
                let mut points = render_split_arc(description, round_fraction, head_base);
                // complete the square part
                points.push(Point {
                    x: cos + side,
                    y: fraction.start * height,
                });
                points.push(Point { x: cos, y: fraction.start * height });
                points
            }
            Not => render_box(description.cell_dim, fraction),
        }
    }
}

fn render_default_curved_segment(
    description: &SegmentDescription,
    mut turn_fraction: f32,
    subsegment_idx: usize,
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
            return render_default_straight_segment(description, subsegment_idx, fraction, round_head);
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

fn render_subsegment(
    description: &SegmentDescription,
    subsegment_idx: usize,
    turn_fraction: f32,
    fraction: SegmentFraction,
    round_head: RoundHeadDescription,
) -> Vec<Point> {
    use TurnDirection::*;
    use TurnType::*;

    let mut segment: ShapePoints;
    match description.turn.turn_type() {
        Straight => {
            // TODO: convert segments to shapes
            segment = render_default_straight_segment(description, subsegment_idx, fraction, round_head).into()
        }
        Blunt(turn_direction) | Sharp(turn_direction) => {
            segment =
                render_default_curved_segment(description, turn_fraction, subsegment_idx, fraction, round_head).into();
            if turn_direction == Clockwise {
                segment = segment.flip_horizontally(description.cell_dim.center().x);
            }
        }
    }

    let rotation_angle = Dir::U.clockwise_angle_to(description.turn.coming_from);
    if rotation_angle != 0. {
        segment = segment.rotate_clockwise(description.cell_dim.center(), rotation_angle);
    }

    segment = segment.translate(description.destination);

    segment.into()
}

impl SegmentRenderer for SmoothSegments {
    fn render_segment(
        description: &SegmentDescription,
        turn_fraction: f32,
        round_head: RoundHeadDescription,
        color_resolution: usize,
    ) -> Box<dyn Iterator<Item = Polygon> + '_> {
        let mut end = description.fraction.start;

        // TODO: in general, this isn't pretty
        Box::new(
            description
                .get_subsegments(color_resolution)
                .map(move |subsegment| {
                    let start = end;
                    end = subsegment.end;
                    (subsegment.subsegment_idx, start, end, subsegment.color)
                })
                .map(move |(subsegment_idx, start, end, color)| {
                    let points = render_subsegment(
                        description,
                        subsegment_idx,
                        turn_fraction,
                        SegmentFraction { start, end },
                        round_head,
                    );
                    Polygon { points, color }
                }),
        )
    }
}
