use std::f32::consts::PI;

use ggez::{
    graphics::{DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use num_integer::Integer;

use crate::{app::palette::GamePalette, basic::*};
use crate::app::snake::drawing::point_factory::{AnimatedSegmentsPointy, PointFactory, AnimatedSegmentsSmooth, HexagonSegments};

pub(crate) mod point_factory;
// mod animated_points_pointy;
// mod animated_points_smooth;

// TODO: make this readable
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
            builder.line(
                #[rustfmt::skip] &[
                    Point { x: cos + dh, y: dv },
                    Point { x: cos + side + dh, y: dv },
                ],
                palette.grid_thickness,
                color,
            )?;

            // line between b and a
            if !(dim.h.is_odd() && h == (dim.h + 1) / 2 - 1) {
                builder.line(
                    #[rustfmt::skip] &[
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
    for point in points.iter_mut() {
        *point = point.clockwise_rotate_around(origin, angle);
    }
}

pub(crate) fn translate(points: &mut [Point], dest: Point) {
    for point in points {
        *point += dest;
    }
}

pub enum SegmentFraction {
    Appearing(f32),
    Disappearing(f32),
    Solid,
}



pub fn get_points_animated(
    dest: Point,
    previous: Dir,
    next: Dir,
    cell_dim: CellDim,
    fraction: SegmentFraction,
    draw_style: DrawStyle,
) -> Vec<Point> {
    let (appear, disappear) = match fraction {
        SegmentFraction::Appearing(f) => (f, 1.),
        SegmentFraction::Disappearing(f) => (1., 1. - f),
        SegmentFraction::Solid => (1., 1.),
    };

    // NOTE: a lot of this function is redundant if only hexagons are being used
    let point_factory: Box<dyn PointFactory> = match draw_style {
        DrawStyle::Hexagon => Box::new(HexagonSegments),
        DrawStyle::Pointy => Box::new(AnimatedSegmentsPointy),
        DrawStyle::Smooth => Box::new(AnimatedSegmentsSmooth),
    };

    let mut points;
    let angle;

    match previous.turn_type(next) {
        TurnType::Straight => {
            points = point_factory.straight_segment(cell_dim, appear, disappear);
            angle = previous.clockwise_angle_from_u();
        }
        TurnType::Blunt(turn_direction) => {
            // feels hacky
            let (ang, appear, disappear) = match turn_direction {
                TurnDirection::Clockwise => (previous.clockwise_angle_from_u(), appear, disappear),
                TurnDirection::CounterClockwise => {
                    (next.clockwise_angle_from_u(), disappear, appear)
                }
            };

            points = point_factory.blunt_turn_segment(cell_dim, appear, disappear);
            angle = ang;
        }
        TurnType::Sharp(turn_direction) => {
            // feels hacky
            let (ang, appear, disappear) = match turn_direction {
                TurnDirection::Clockwise => (previous.clockwise_angle_from_u(), appear, disappear),
                TurnDirection::CounterClockwise => {
                    (next.clockwise_angle_from_u(), disappear, appear)
                }
            };

            points = point_factory.sharp_turn_segment(cell_dim, appear, disappear);
            angle = ang;
        }
    }

    rotate(&mut points, angle, cell_dim.center());
    translate(&mut points, dest);
    points
}
