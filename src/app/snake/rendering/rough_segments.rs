use crate::{
    app::snake::rendering::{descriptions::SegmentFraction, point_factory::SegmentRenderer},
    basic::{CellDim, Point},
};
use crate::app::snake::rendering::descriptions::{SegmentDescription, TurnType};
use std::f32::consts::FRAC_PI_3;

pub struct RoughSegments;

impl SegmentRenderer for RoughSegments {
    fn render_straight_segment(description: &SegmentDescription) -> Vec<Point> {
        let CellDim { side, sin, cos } = description.cell_dim;
        let SegmentFraction { start, end } = description.fraction;

        // top-left, top-right, bottom-right, bottom-left
        vec![
            Point { x: cos, y: end * 2. * sin },
            Point { x: cos + side, y: end * 2. * sin },
            Point { x: cos + side, y: start * 2. * sin },
            Point { x: cos, y: start * 2. * sin },
        ]
    }

    fn render_curved_segment(description: &SegmentDescription, mut turn: f32) -> Vec<Point> {
        if let TurnType::Blunt(_) = description.turn.turn_type() {
            turn /= 2.;
        }

        let total_angle = turn * 2. * FRAC_PI_3;

        let SegmentDescription {
            cell_dim,
            fraction,
            ..
        } = description;

        let CellDim { side, sin: _, cos } = *cell_dim;

        // distance of the pivot from where it is for a sharp turn
        let pivot_dist = 2. * cos * (1. / turn - 1.);
        // too straight to be drawn as curved, default to straight drawing
        if pivot_dist.is_infinite() {
            return Self::render_straight_segment(description);
        }
        let pivot = Point { x: side + cos + pivot_dist, y: 0. };

        let inner_line_start = Point { x: cos + side, y: 0. };
        let inner_line_end = inner_line_start.rotate_counterclockwise(pivot, total_angle);
        let outer_line_start = Point { x: cos, y: 0. };
        let outer_line_end = outer_line_start.rotate_counterclockwise(pivot, total_angle);

        // TODO: reintroduce elbow point for sharp turns
        vec![
            (1. - fraction.start) * inner_line_start + fraction.start * inner_line_end,
            (1. - fraction.start) * outer_line_start + fraction.start * outer_line_end,
            (1. - fraction.end) * outer_line_start + fraction.end * outer_line_end,
            (1. - fraction.end) * inner_line_start + fraction.end * inner_line_end,
        ]
    }

    // fn render_default_sharp(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
    //     let CellDim { side, sin, cos } = cell_dim;
    //     let SegmentFraction { start, end } = fraction;
    //
    //     // The point around which the segment 'rotates'
    //     let pivot = Point { x: cos + side, y: 0. };
    //
    //     // The three points it touches as it goes around
    //     // the first half of the animation is from a to b,
    //     // the second half is from b to c
    //     let a = Point { x: cos, y: 0. };
    //     let b = cell_dim.center();
    //     let c = Point { x: 2. * cos + side, y: sin };
    //
    //     let mut points = Vec::with_capacity(4);
    //
    //     points.push(pivot);
    //     if end >= 0.5 {
    //         let end = (end - 0.5) / 0.5;
    //         points.push((1. - end) * b + end * c);
    //         if start < 0.5 {
    //             let start = start / 0.5;
    //             points.push(b);
    //             points.push((1. - start) * a + start * b);
    //         } else {
    //             let start = (start - 0.5) / 0.5;
    //             points.push((1. - start) * b + start * c);
    //         }
    //     } else {
    //         let end = end / 0.5;
    //         points.push((1. - end) * a + end * b);
    //         // assume start <= end, so start < 0.5
    //         let start = start / 0.5;
    //         points.push((1. - start) * a + start * b);
    //     }
    //
    //     points
    // }
}
