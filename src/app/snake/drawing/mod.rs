use ggez::{
    graphics::{DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};
use num_integer::Integer;

use crate::{
    app::{
        palette::GamePalette,
        // snake::drawing::point_factory::{
        //     AnimatedSegmentsPointy, AnimatedSegmentsSmooth, HexagonSegments, PointFactory,
        // },
    },
    basic::*,
};

pub(crate) use point_factory::translate;

pub mod point_factory;
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
