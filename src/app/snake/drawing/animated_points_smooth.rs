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
    CellDim { side, cos, .. }: CellDim,
    show_front: f32,
    show_back: f32,
) -> Vec<Point> {
    let pivot = #[rustfmt::skip] Point { x: side + 3. * cos, y: 0. };
    let a = #[rustfmt::skip] Point { x: cos, y: 0. };
    let b = #[rustfmt::skip] Point { x: side + cos, y: 0. };

    let mut points = vec![];
    let angle_start = show_back * (-PI / 3.);
    let angle_end = (1. - show_front) * (-PI / 3.);

    // a one way
    let mut ang = angle_start;
    while ang < angle_end {
        let pt = a.clockwise_rotate_around(pivot, ang);
        points.push(pt);
        ang += ANGLE_STEP;
    }
    points.push(a.clockwise_rotate_around(pivot, angle_end));

    // b the other way
    ang = angle_end;
    while ang > angle_start {
        let pt = b.clockwise_rotate_around(pivot, ang);
        points.push(pt);
        ang -= ANGLE_STEP;
    }
    points.push(b.clockwise_rotate_around(pivot, angle_start));

    // hacky
    if points.len() < 3 {
        assert_eq!(points.len(), 2);
        points.push(pivot);
    }

    points
}

// UR => U
pub fn sharp_turn_segment(
    CellDim { side, cos, .. }: CellDim,
    show_front: f32,
    show_back: f32,
) -> Vec<Point> {
    let pivot = #[rustfmt::skip] Point { x: side + cos, y: 0. };
    let a = #[rustfmt::skip] Point { x: cos, y: 0. };

    let mut points = vec![];
    points.push(pivot);

    let angle_start = show_back * (-2. * PI / 3.);
    let angle_end = (1. - show_front) * (-2. * PI / 3.);

    let mut ang = angle_start;
    while ang < angle_end {
        let pt = a.clockwise_rotate_around(pivot, ang);
        points.push(pt);
        ang += ANGLE_STEP;
    }
    points.push(a.clockwise_rotate_around(pivot, angle_end));

    // hacky
    if points.len() < 3 {
        assert_eq!(points.len(), 2);
        points.push(pivot);
    }

    points
}
