use crate::{
    app::snake::rendering::{
        descriptions::SegmentFraction, point_factory::SegmentRenderer,
        rough_segments::RoughSegments,
    },
    basic::{CellDim, Point},
};
use std::cmp::max;

pub struct SmoothSegments;

const NUM_ANGLE_SEGMENTS: usize = 3;

impl SegmentRenderer for SmoothSegments {
    fn render_default_straight(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
        RoughSegments::render_default_straight(cell_dim, fraction)
    }

    fn render_default_blunt(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
        use std::f32::consts::FRAC_PI_3;

        let CellDim { side, sin, cos } = cell_dim;
        let SegmentFraction { start, end } = fraction;

        let pivot = Point { x: side + 3. * cos, y: 0. };
        let inner_line_start = Point { x: cos + side, y: 0. };
        let outer_line_start = Point { x: cos, y: 0. };

        let fraction_size = fraction.end - fraction.start;
        let num_angle_segments = max(1, (NUM_ANGLE_SEGMENTS as f32 * fraction_size) as usize);

        let mut points = Vec::with_capacity(num_angle_segments * 2 + 2);
        let start_angle = start * FRAC_PI_3;
        let end_angle = end * FRAC_PI_3;

        let inner_line = (0..=num_angle_segments).map(move |i| {
            let i = i as f32 / num_angle_segments as f32;
            let angle = (1. - i) * start + i * end_angle;
            inner_line_start.rotate_counterclockwise(pivot, angle)
        });

        // outer line in opposite direction (notice call to rev)
        let outer_line = (0..=num_angle_segments).rev().map(move |i| {
            let i = i as f32 / num_angle_segments as f32;
            let angle = (1. - i) * start_angle + i * end_angle;
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

    fn render_default_sharp(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point> {
        use std::f32::consts::FRAC_PI_3;

        let CellDim { side, sin, cos } = cell_dim;
        let SegmentFraction { start, end } = fraction;

        let pivot = Point { x: side + cos, y: 0. };
        let line_start = Point { x: cos, y: 0. };

        let fraction_size = fraction.end - fraction.start;
        let num_angle_segments = max(1, (NUM_ANGLE_SEGMENTS as f32 * fraction_size) as usize);

        let mut points = Vec::with_capacity(num_angle_segments * 2 + 2 + 1); // + pivot
        let start_angle = start * 2. * FRAC_PI_3;
        let end_angle = end * 2. * FRAC_PI_3;

        points.push(pivot);

        let line = (0..=num_angle_segments).rev().map(move |i| {
            let i = i as f32 / num_angle_segments as f32;
            let angle = (1. - i) * start_angle + i * end_angle;
            line_start.rotate_counterclockwise(pivot, angle)
        });

        points.extend(line);

        // hacky
        if points.len() < 3 {
            assert_eq!(points.len(), 2);
            points.push((*points.last().unwrap() + *points.first().unwrap()) / 2.);
        }

        points
    }
}
