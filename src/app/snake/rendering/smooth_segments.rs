use crate::{
    app::snake::rendering::{
        descriptions::SegmentFraction, point_factory::SegmentRenderer,
    },
    basic::{CellDim, Point},
};
use std::cmp::max;
use crate::app::snake::rendering::descriptions::{SegmentDescription, TurnType};
use std::f32::consts::FRAC_PI_3;

pub struct SmoothSegments;

const NUM_ANGLE_SEGMENTS: usize = 10;

impl SegmentRenderer for SmoothSegments {
    fn render_straight_segment(description: &SegmentDescription) -> Vec<Point> {
        let SegmentDescription {
            cell_dim,
            fraction,
            ..
        } = description;

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
        let outer_line_start = Point { x: cos, y: 0. };

        let fraction_size = fraction.end - fraction.start;
        let num_angle_segments = max(1, (NUM_ANGLE_SEGMENTS as f32 * fraction_size) as usize);

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

        // hacky
        if points.len() < 3 {
            assert_eq!(points.len(), 2);
            points.push((*points.last().unwrap() + *points.first().unwrap()) / 2.);
        }

        points
    }
}
