use crate::basic::{CellDim, Dir, DrawStyle, Point, TurnDirection, TurnType};
use std::f32::consts::PI;

pub enum SegmentFraction {
    Appearing(f32),
    Disappearing(f32),
    Solid,
}

// pub enum Turn {
//     Straight {
//         previous: Dir,
//     },
//     NormalTurn {
//         previous: Dir,
//         next: Dir,
//     },
//     TransitionTurn {
//         previous: Dir,
//         next1: Dir,
//         next2: Dir,
//     },
// }

fn rotate(points: &mut [Point], angle: f32, origin: Point) {
    for point in points.iter_mut() {
        *point = point.clockwise_rotate_around(origin, angle);
    }
}

pub fn translate(points: &mut [Point], dest: Point) {
    for point in points {
        *point += dest;
    }
}

// // the trait includes only clockwise turns, for counterclockwise turns, show_front and show_back are reversed
// pub trait PointFactory {
//     // D => U
//     fn straight_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point>;
//
//     // D => UR
//     fn blunt_turn_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point>;
//
//     // D => DR
//     fn sharp_turn_segment(&self, cell_dim: CellDim, show_front: f32, show_back: f32) -> Vec<Point>;
// }

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

pub struct PointySegments;

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

impl PointySegments {
    fn blunt_turn_segment(
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
        CellDim { side, sin, cos }: CellDim,
        show_front: f32,
        show_back: f32,
    ) -> Vec<Point> {
        let pivot = Point { x: side + cos, y: 0. };
        let a = Point { x: cos, y: 0. };
        // let b = #[rustfmt::skip] Point { x: cos, y: 2. * sin };
        let b = Point { x: cos + side / 2., y: sin };
        let c = Point { x: side + 2. * cos, y: sin };

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

pub struct SmoothSegments;

impl SmoothSegments {
    const ANGLE_STEP: f32 = 0.1;

    fn blunt_turn_segment(
        CellDim { side, cos, .. }: CellDim,
        show_front: f32,
        show_back: f32,
    ) -> Vec<Point> {
        let pivot = Point { x: side + 3. * cos, y: 0. };
        let a = Point { x: cos, y: 0. };
        let b = Point { x: side + cos, y: 0. };

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
        CellDim { side, cos, .. }: CellDim,
        show_front: f32,
        show_back: f32,
    ) -> Vec<Point> {
        let pivot = Point { x: side + cos, y: 0. };
        let a = Point { x: cos, y: 0. };

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

pub struct SegmentDescription {
    pub location: Point,
    pub previous_segment: Dir,
    pub next_segment: Dir,
    pub fraction: SegmentFraction,
    pub draw_style: DrawStyle,

    pub cell_dim: CellDim,
}

impl SegmentDescription {
    pub fn render(self) -> Vec<Point> {
        match self.draw_style {
            DrawStyle::Hexagon => HexagonSegments::render_segment(self),
            DrawStyle::Pointy => PointySegments::render_segment(self),
            DrawStyle::Smooth => SmoothSegments::render_segment(self),
        }
    }
}

pub trait SegmentRenderer {
    fn render_segment(description: SegmentDescription) -> Vec<Point>;
}

impl SegmentRenderer for HexagonSegments {
    fn render_segment(description: SegmentDescription) -> Vec<Point> {
        let mut points = full_hexagon(description.cell_dim);
        translate(&mut points, description.location);
        points
    }
}

macro_rules! implement_segment_renderer {
    ($renderer:ident) => {
        impl SegmentRenderer for $renderer {
            fn render_segment(description: SegmentDescription) -> Vec<Point> {
                use TurnDirection::*;
                use TurnType::*;

                let (mut show_front, mut show_back) = match description.fraction {
                    SegmentFraction::Appearing(f) => (f, 1.),
                    SegmentFraction::Disappearing(f) => (1., 1. - f),
                    SegmentFraction::Solid => (1., 1.),
                };

                let turn_type = description
                    .previous_segment
                    .turn_type(description.next_segment);
                let angle = match turn_type {
                    Blunt(Clockwise) | Sharp(Clockwise) => {
                        description.previous_segment.clockwise_angle_from_u()
                    }
                    Blunt(CounterClockwise) | Sharp(CounterClockwise) | Straight => {
                        std::mem::swap(&mut show_front, &mut show_back);
                        description.next_segment.clockwise_angle_from_u()
                    }
                };
                let mut points = match turn_type {
                    Straight => thin_straight_segment(description.cell_dim, show_front, show_back),
                    Blunt(_) => {
                        Self::blunt_turn_segment(description.cell_dim, show_front, show_back)
                    }
                    Sharp(_) => {
                        Self::sharp_turn_segment(description.cell_dim, show_front, show_back)
                    }
                };

                rotate(&mut points, angle, description.cell_dim.center());
                translate(&mut points, description.location);
                points
            }
        }
    };
}

implement_segment_renderer!(PointySegments);
implement_segment_renderer!(SmoothSegments);
