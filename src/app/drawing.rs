use crate::{
    app::{
        game::CellDim,
        hex::{Dir, HexDim, TurnDirection, TurnType},
        palette::GamePalette,
    },
    point::Point,
};
use ggez::{
    graphics::{DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use num_integer::Integer;

pub fn generate_grid_mesh(
    ctx: &mut Context,
    dim: HexDim,
    palette: &GamePalette,
    cell_dim: CellDim,
) -> GameResult<Mesh> {
    let CellDim { side, sin, cos } = cell_dim;

    // two kinds of alternating vertical lines
    let mut vline_a = vec![];
    let mut vline_b = vec![];

    #[rustfmt::skip]
    for dv in (0..=dim.v).map(|v| v as f32 * 2. * sin) {
        vline_a.push(Point { x: cos, y: dv });
        vline_a.push(Point { x: 0., y: dv + sin });
        vline_b.push(Point { x: cos + side, y: dv });
        vline_b.push(Point { x: 2. * cos + side, y: dv + sin });
    }

    let mut builder = MeshBuilder::new();

    let draw_mode = DrawMode::stroke(palette.grid_thickness);
    let color = palette.grid_color;
    for h in 0..(dim.h + 1) / 2 {
        if h == 0 {
            builder.polyline(draw_mode, &vline_a[..vline_a.len() - 1], color)?;
        } else {
            builder.polyline(draw_mode, &vline_a, color)?;
        }
        if dim.h.is_odd() && h == (dim.h + 1) / 2 - 1 {
            builder.polyline(draw_mode, &vline_b[..vline_b.len() - 1], color)?;
        } else {
            builder.polyline(draw_mode, &vline_b, color)?;
        }

        let dh = h as f32 * (2. * side + 2. * cos);

        for v in 0..=dim.v {
            let dv = v as f32 * 2. * sin;

            // line between a and b
            #[rustfmt::skip]
            builder.line(
                &[
                    Point { x: cos + dh, y: dv },
                    Point { x: cos + side + dh, y: dv },
                ],
                palette.grid_thickness,
                color,
            )?;

            // line between b and a
            if !(dim.h.is_odd() && h == (dim.h + 1) / 2 - 1) {
                #[rustfmt::skip]
                builder.line(
                    &[
                        Point { x: 2. * cos + side + dh, y: sin + dv },
                        Point { x: 2. * cos + 2. * side + dh, y: sin + dv },
                    ],
                    palette.grid_thickness,
                    color,
                )?;
            }
        }

        // shift the lines right by 2 cells
        let offset = 2. * (side + cos);
        vline_a.iter_mut().for_each(|a| a.x += offset);
        vline_b.iter_mut().for_each(|b| b.x += offset);
    }
    if dim.h.is_even() {
        builder.polyline(draw_mode, &vline_a[1..], color)?;
    }

    builder.build(ctx)
}

pub fn generate_border_mesh(ctx: &mut Context) -> GameResult<Mesh> {
    let _ = ctx;
    unimplemented!()
}

fn rotate(points: &mut [Point], angle: f32, origin: Point) {
    let sin = angle.sin();
    let cos = angle.cos();

    for point in points.iter_mut() {
        *point -= origin;
        *point = Point {
            x: point.x * cos - point.y * sin,
            y: point.x * sin + point.y * cos,
        };
        *point += origin;
    }
}

fn translate(points: &mut [Point], dest: Point) {
    for point in points {
        *point += dest;
    }
}

pub enum SegmentFraction {
    Appearing(f32),
    Disappearing(f32),
    Solid,
}

pub fn get_full_hexagon(dest: Point, cell_dim: CellDim) -> Vec<Point> {
    let CellDim { side, sin, cos } = cell_dim;

    let mut points = #[rustfmt::skip] vec![
        Point { x: cos, y: 0. },
        Point { x: side + cos, y: 0. },
        Point { x: side + 2. * cos, y: sin },
        Point { x: side + cos, y: 2. * sin },
        Point { x: cos, y: 2. * sin },
        Point { x: 0., y: sin },
    ];

    translate(&mut points, dest);
    points
}

pub fn get_points_animated(
    dest: Point,
    previous: Dir,
    next: Dir,
    cell_dim: CellDim,
    fraction: SegmentFraction,
) -> Vec<Point> {
    // higher appear => more revealed of front
    // higher disappear => more revealed of back
    // if both are 1, segment  is fully revealed
    let (appear, disappear) = match fraction {
        SegmentFraction::Appearing(f) => (f, 1.),
        SegmentFraction::Disappearing(f) => (1., 1. - f),
        SegmentFraction::Solid => (1., 1.),
    };

    let CellDim { side, sin, cos } = cell_dim;

    let mut points;
    match previous.turn_type(next) {
        TurnType::Straight => {
            // D => U
            points = #[rustfmt::skip] vec![
                Point { x: cos, y: 2. * sin * (1. - appear) },
                Point { x: side + cos, y: 2. * sin * (1. - appear) },
                Point { x: side + cos, y: 2. * sin * disappear },
                Point { x: cos, y: 2. * sin * disappear },
            ];

            rotate(
                &mut points,
                previous.clockwise_angle_from_u(),
                cell_dim.center(),
            );
        }
        TurnType::Blunt(turn_direction) => {
            // DR => U
            // points = #[rustfmt::skip] vec![
            //     Point { x: cos, y: 0. },
            //     Point { x: side + cos, y: 0. },
            //     Point { x: side + 2. * cos, y: sin },
            //     Point { x: side + cos, y: 2. * sin },
            // ];

            // feels hacky
            let (angle, appear, disappear) = match turn_direction {
                TurnDirection::Clockwise => (previous.clockwise_angle_from_u(), appear, disappear),
                TurnDirection::CounterClockwise => {
                    (next.clockwise_angle_from_u(), disappear, appear)
                }
            };

            let start_a = #[rustfmt::skip] Point { x: side + cos, y: 2. * sin };
            let start_b = #[rustfmt::skip] Point { x: side + 2. * cos, y: sin };
            let end_a = #[rustfmt::skip] Point { x: cos, y: 0. };
            let end_b = #[rustfmt::skip] Point { x: side + cos, y: 0. };

            points = vec![
                appear * end_a + (1. - appear) * start_a,
                appear * end_b + (1. - appear) * start_b,
                disappear * start_b + (1. - disappear) * end_b,
                disappear * start_a + (1. - disappear) * end_a,
            ];

            rotate(&mut points, angle, cell_dim.center());
        }
        TurnType::Sharp(turn_direction) => {
            // UR => U
            // points = #[rustfmt::skip] vec![
            //     Point { x: cos, y: 0. },
            //     Point { x: side + cos, y: 0. },
            //     Point { x: side + 2. * cos, y: sin },
            //     Point { x: cos, y: 2. * sin },
            // ];

            let (angle, appear, disappear) = match turn_direction {
                TurnDirection::Clockwise => (previous.clockwise_angle_from_u(), appear, disappear),
                TurnDirection::CounterClockwise => {
                    (next.clockwise_angle_from_u(), disappear, appear)
                }
            };

            assert!(
                appear == 1. || disappear == 1.,
                "simultaneous appar and disappear not implemented"
            );

            let pivot = #[rustfmt::skip] Point { x: side + cos, y: 0. };
            let a = #[rustfmt::skip] Point { x: cos, y: 0. };
            let b = #[rustfmt::skip] Point { x: cos, y: 2. * sin };
            let c = #[rustfmt::skip] Point { x: side + 2. * cos, y: sin };

            points = Vec::with_capacity(4);
            if appear < 1. {
                if appear >= 0.5 {
                    let f = (appear - 0.5) / 0.5;
                    points.push(f * a + (1. - f) * b);
                    points.push(b);
                } else {
                    let f = appear / 0.5;
                    points.push(f * b + (1. - f) * c);
                }
                points.push(c);
                points.push(pivot);
            } else {
                points.push(pivot);
                points.push(a);
                if disappear >= 0.5 {
                    let f = (disappear - 0.5) / 0.5;
                    points.push(b);
                    points.push((1. - f) * b + f * c);
                } else {
                    let f = disappear / 0.5;
                    points.push((1. - f) * a + f * b);
                }
            }

            rotate(&mut points, angle, cell_dim.center());
        }
    }
    translate(&mut points, dest);
    points
}
