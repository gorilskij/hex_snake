use crate::{
    app::snake::palette::SegmentStyle,
    basic::{CellDim, Dir, DrawStyle, Point, TurnDirection, TurnType},
};
use ggez::{
    graphics::{Color, DrawMode, MeshBuilder},
    GameResult,
};
use hsl::HSL;
use itertools::Itertools;
use std::{cmp::max, f32::consts::PI};

// a full segment starts at 0. and ends at 1.
pub struct SegmentFraction {
    pub start: f32,
    pub end: f32,
}

impl SegmentFraction {
    pub fn solid() -> Self {
        Self { start: 0., end: 1. }
    }

    pub fn appearing(f: f32) -> Self {
        Self { start: 0., end: f }
    }

    pub fn disappearing(f: f32) -> Self {
        Self { start: f, end: 1. }
    }
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
    pub segment_style: SegmentStyle,

    pub cell_dim: CellDim,
}

impl SegmentDescription {
    const SUBSEGMENT_STEPS: usize = 4;

    // for simulating gradient
    fn split_into_subsegments(self) -> Vec<Self> {
        let segment_size = self.fraction.end - self.fraction.start;
        let subsegment_steps = max(2, (Self::SUBSEGMENT_STEPS as f32 * segment_size) as usize);

        let mut colors = match self.segment_style {
            SegmentStyle::Solid(color) => vec![color],
            SegmentStyle::RGBGradient {
                start_rgb: (r1, g1, b1),
                end_rgb: (r2, g2, b2),
            } => {
                let r_iter = itertools_num::linspace(r1 as f64, r2 as f64, subsegment_steps);
                let g_iter = itertools_num::linspace(g1 as f64, g2 as f64, subsegment_steps);
                let b_iter = itertools_num::linspace(b1 as f64, b2 as f64, subsegment_steps);
                r_iter
                    .zip(g_iter)
                    .zip(b_iter)
                    .map(|((r, g), b)| Color::from_rgb(r as u8, g as u8, b as u8))
                    .collect()
            }
            SegmentStyle::HSLGradient { start_hue, end_hue, lightness } => {
                itertools_num::linspace(start_hue, end_hue, subsegment_steps)
                    .map(|hue| {
                        let hsl = HSL { h: hue, s: 1., l: lightness };
                        Color::from(hsl.to_rgb())
                    })
                    .collect()
            }
        };

        let len1 = colors.len();
        colors.dedup();
        assert_eq!(colors.len(), len1, "generated duplicate colors");

        colors.pop();

        let num_subsegments = colors.len();
        let subsegment_size = segment_size / num_subsegments as f32;

        colors
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, color)| {
                let start = self.fraction.start + subsegment_size * i as f32;
                let end = start + subsegment_size;
                Self {
                    location: self.location,
                    previous_segment: self.previous_segment,
                    next_segment: self.next_segment,
                    fraction: SegmentFraction { start, end },
                    draw_style: self.draw_style,
                    segment_style: SegmentStyle::Solid(color),
                    cell_dim: self.cell_dim,
                }
            })
            .collect_vec()
    }

    // subsegments in particular are expected to be of the Solid variant
    fn unwrap_solid_color(&self) -> Color {
        match &self.segment_style {
            SegmentStyle::Solid(color) => *color,
            style => panic!("Tried to unwrap {:?} as Solid", self.segment_style),
        }
    }

    pub fn render(mut self) -> Vec<(Color, Vec<Point>)> {
        let subsegments = if let DrawStyle::Hexagon = self.draw_style {
            // hexagon segments don't support gradients
            self.fraction = SegmentFraction::solid();
            self.segment_style = self.segment_style.into_solid();
            vec![self]
        } else {
            self.split_into_subsegments()
        };

        subsegments
            .into_iter()
            .map(|subsegment| {
                let color = subsegment.unwrap_solid_color();
                let points = match subsegment.draw_style {
                    DrawStyle::Hexagon => HexagonSegments::render_segment(subsegment),
                    DrawStyle::Pointy => PointySegments::render_segment(subsegment),
                    DrawStyle::Smooth => SmoothSegments::render_segment(subsegment),
                };
                (color, points)
            })
            .collect()
    }

    pub fn build(self, builder: &mut MeshBuilder) -> GameResult {
        for (color, points) in self.render() {
            builder.polygon(DrawMode::fill(), &points, color)?;
        }
        Ok(())
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

// impl SegmentRenderer for PointySegments {
//     fn render_segment(description: SegmentDescription) -> Vec<(Color, Vec<Point>)> {
//         unimplemented!()
//     }
// }
//
// impl SegmentRenderer for SmoothSegments {
//     fn render_segment(description: SegmentDescription) -> Vec<(Color, Vec<Point>)> {
//         use TurnDirection::*;
//         use TurnType::*;
//
//         let (mut show_front, mut show_back) = match description.fraction {
//             SegmentFraction::Appearing(f) => (f, 1.),
//             SegmentFraction::Disappearing(f) => (1., 1. - f),
//             SegmentFraction::Solid => (1., 1.),
//         };
//
//         let turn_type = description
//             .previous_segment
//             .turn_type(description.next_segment);
//         let angle = match turn_type {
//             Blunt(Clockwise) | Sharp(Clockwise) => {
//                 description.previous_segment.clockwise_angle_from_u()
//             }
//             Blunt(CounterClockwise) | Sharp(CounterClockwise) | Straight => {
//                 std::mem::swap(&mut show_front, &mut show_back);
//                 description.next_segment.clockwise_angle_from_u()
//             }
//         };
//         let mut points = match turn_type {
//             Straight => thin_straight_segment(description.cell_dim, show_front, show_back),
//             Blunt(_) => {
//                 Self::blunt_turn_segment(description.cell_dim, show_front, show_back)
//             }
//             Sharp(_) => {
//                 Self::sharp_turn_segment(description.cell_dim, show_front, show_back)
//             }
//         };
//
//         rotate(&mut points, angle, description.cell_dim.center());
//         translate(&mut points, description.location);
//         points
//     }
// }

macro_rules! implement_segment_renderer {
    ($renderer:ident) => {
        impl SegmentRenderer for $renderer {
            fn render_segment(description: SegmentDescription) -> Vec<Point> {
                use TurnDirection::*;
                use TurnType::*;

                // let (mut show_front, mut show_back) = match description.fraction {
                //     SegmentFraction::Appearing(f) => (f, 1.),
                //     SegmentFraction::Disappearing(f) => (1., 1. - f),
                //     SegmentFraction::Solid => (1., 1.),
                // };
                let mut show_front = description.fraction.end;
                let mut show_back = 1. - description.fraction.start;

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
