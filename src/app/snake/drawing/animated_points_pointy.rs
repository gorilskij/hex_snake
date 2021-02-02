use std::f32::consts::PI;

use crate::{
    app::snake::drawing::SegmentFraction,
    basic::{CellDim, Point},
};

// show_front: fraction of the block starting at the back to show
// show_back: fraction of the block starting at the front to show
// as in show_front < 1. at the front of the snake and show_back < 1. at the back of the snake
// invariant: show_front + show_back > 1., otherwise the segment would not display
// TODO: might make more sense to talk of hide_front and hide_back

// for curved segments
const ANGLE_STEP: f32 = 0.1;

// D => U
pub fn straight_segment(
    CellDim { side, sin, cos }: CellDim,
    show_front: f32,
    show_back: f32,
) -> Vec<Point> {
    #[rustfmt::skip] vec![
        Point { x: cos,        y: 2. * sin * (1. - show_front) },
        Point { x: cos + side, y: 2. * sin * (1. - show_front) },
        Point { x: cos + side, y: 2. * sin * show_back },
        Point { x: cos,        y: 2. * sin * show_back },
    ]
}

// DR => U
pub fn blunt_turn_segment(
    CellDim { side, sin, cos }: CellDim,
    show_front: f32,
    show_back: f32,
) -> Vec<Point> {
    let start_a = #[rustfmt::skip] Point { x: side + cos, y: 2. * sin };
    let start_b = #[rustfmt::skip] Point { x: side + 2. * cos, y: sin };
    let end_a = #[rustfmt::skip] Point { x: cos, y: 0. };
    let end_b = #[rustfmt::skip] Point { x: side + cos, y: 0. };

    #[rustfmt::skip] vec![
        show_front * end_a   + (1. - show_front) * start_a,
        show_front * end_b   + (1. - show_front) * start_b,
        show_back  * start_b + (1. - show_back)  * end_b,
        show_back  * start_a + (1. - show_back)  * end_a,
    ]
}

// UR => U
pub fn sharp_turn_segment(
    CellDim { side, sin, cos }: CellDim,
    show_front: f32,
    show_back: f32,
) -> Vec<Point> {
    let pivot = #[rustfmt::skip] Point { x: side + cos, y: 0. };
    let a = #[rustfmt::skip] Point { x: cos, y: 0. };
    let b = #[rustfmt::skip] Point { x: cos, y: 2. * sin };
    let c = #[rustfmt::skip] Point { x: side + 2. * cos, y: sin };

    let mut points = Vec::with_capacity(4);

    if show_front < 1. {
        if show_front >= 0.5 {
            let f = (show_front - 0.5) / 0.5;
            points.push(f * a + (1. - f) * b);
            points.push(b);
        } else {
            let f = show_front / 0.5;
            points.push(f * b + (1. - f) * c);
        }
        points.push(c);
        points.push(pivot);
    } else {
        if show_back >= 0.5 {
            let f = (show_back - 0.5) / 0.5;
            points.push(b);
            points.push((1. - f) * b + f * c);
        } else {
            let f = show_back / 0.5;
            points.push((1. - f) * a + f * b);
        }
        points.push(pivot);
        points.push(a);
    }

    points
}
