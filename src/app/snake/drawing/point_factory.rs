use crate::basic::{CellDim, Point};
use std::f32::consts::PI;

pub trait PointFactory {
    // D => U
    fn straight_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point>;

    // DR => U
    fn blunt_turn_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point>;

    // UR => U
    fn sharp_turn_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point>;
}

pub struct HexagonSegments;

pub(crate) fn full_hexagon(CellDim { side, sin, cos }: CellDim) -> Vec<Point> {
    #[rustfmt::skip] vec![
        Point { x: cos,             y: 0. },
        Point { x: cos + side,      y: 0. },
        Point { x: cos * 2. + side, y: sin },
        Point { x: cos + side,      y: sin * 2. },
        Point { x: cos,             y: sin * 2. },
        Point { x: 0.,              y: sin },
    ]
}

impl PointFactory for HexagonSegments {
    fn straight_segment(&self, cell_dim: CellDim, _: f32, _: f32) -> Vec<Point> {
        full_hexagon(cell_dim)
    }

    fn blunt_turn_segment(&self, cell_dim: CellDim, _: f32, _: f32) -> Vec<Point> {
        full_hexagon(cell_dim)
    }

    fn sharp_turn_segment(&self, cell_dim: CellDim, _: f32, _: f32) -> Vec<Point> {
        full_hexagon(cell_dim)
    }
}

pub struct AnimatedSegmentsPointy;

// D => U
fn thin_straight_segment(
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

impl PointFactory for AnimatedSegmentsPointy {
    fn straight_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point> {
        thin_straight_segment(cell_dim, show_front, show_back)
    }

    fn blunt_turn_segment(
        &self,
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

    fn sharp_turn_segment(
        &self,
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
}

pub struct AnimatedSegmentsSmooth;

impl AnimatedSegmentsSmooth {
    const ANGLE_STEP: f32 = 0.1;
}

impl PointFactory for AnimatedSegmentsSmooth {
    fn straight_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point> {
        thin_straight_segment(cell_dim, show_front, show_back)
    }

    fn blunt_turn_segment(
        &self,
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
            ang += Self::ANGLE_STEP;
        }
        points.push(a.clockwise_rotate_around(pivot, angle_end));

        // b the other way
        ang = angle_end;
        while ang > angle_start {
            let pt = b.clockwise_rotate_around(pivot, ang);
            points.push(pt);
            ang -= Self::ANGLE_STEP;
        }
        points.push(b.clockwise_rotate_around(pivot, angle_start));

        // hacky
        if points.len() < 3 {
            assert_eq!(points.len(), 2);
            points.push(pivot);
        }

        points
    }

    fn sharp_turn_segment(
        &self,
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
            ang += Self::ANGLE_STEP;
        }
        points.push(a.clockwise_rotate_around(pivot, angle_end));

        // hacky
        if points.len() < 3 {
            assert_eq!(points.len(), 2);
            points.push(pivot);
        }

        points
    }
}
