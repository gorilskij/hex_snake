use ggez::graphics::{DrawMode, Mesh, MeshBuilder};
use ggez::Context;
use num_integer::Integer;

use crate::app::game_context::GameContext;
use crate::basic::{CellDim, HexDim, Point};
use crate::error::{Error, ErrorConversion, Result};

// TODO: make this readable
// TODO: add option to exclude border from grid mesh
//  when border is drawn separately
pub fn grid_mesh(gtx: &GameContext, ctx: &Context) -> Result<Mesh> {
    let CellDim { side, sin, cos } = gtx.cell_dim;
    let HexDim { h: board_h, v: board_v } = gtx.board_dim;

    // two kinds of alternating vertical lines
    let mut vline_a = vec![]; // lines that start from the top with /
    let mut vline_b = vec![]; // lines that start from the top with \

    #[rustfmt::skip]
    for dv in (0..=board_v).map(|v| v as f32 * 2. * sin) {
        vline_a.push(Point { x: cos, y: dv });
        vline_a.push(Point { x: 0., y: dv + sin });
        vline_b.push(Point { x: cos + side, y: dv });
        vline_b.push(Point { x: 2. * cos + side, y: dv + sin });
    }

    let mut builder = MeshBuilder::new();

    let draw_mode = DrawMode::stroke(gtx.palette.grid_thickness);
    let color = gtx.palette.grid_color;

    let res: Result<_> = try {
        for h in 0..(board_h + 1) / 2 {
            if h == 0 {
                builder.polyline(draw_mode, &vline_a[..vline_a.len() - 1], color)?;
            } else {
                builder.polyline(draw_mode, &vline_a, color)?;
            }
            if board_h.is_odd() && h == (board_h + 1) / 2 - 1 {
                builder.polyline(draw_mode, &vline_b[..vline_b.len() - 1], color)?;
            } else {
                builder.polyline(draw_mode, &vline_b, color)?;
            }

            let dh = h as f32 * 2. * (side + cos);

            for v in 0..=board_v {
                let dv = v as f32 * 2. * sin;

                // line between a and b
                builder.line(
                    #[rustfmt::skip] &[
                        Point { x: cos + dh, y: dv },
                        Point { x: cos + side + dh, y: dv },
                    ],
                    gtx.palette.grid_thickness,
                    color,
                )?;

                // line between b and a
                if !(board_h.is_odd() && h == (board_h + 1) / 2 - 1) {
                    builder.line(
                        #[rustfmt::skip] &[
                            Point { x: 2. * cos + side + dh, y: sin + dv },
                            Point { x: 2. * cos + 2. * side + dh, y: sin + dv },
                        ],
                        gtx.palette.grid_thickness,
                        color,
                    )?;
                }
            }

            // shift the lines right by 2 cells
            let offset = 2. * (side + cos);
            vline_a.iter_mut().for_each(|a| a.x += offset);
            vline_b.iter_mut().for_each(|b| b.x += offset);
        }
        if board_h.is_even() {
            builder.polyline(draw_mode, &vline_a[1..], color)?;
        }

        builder.build()
    };
    // TODO: write a proc macro that does this with an #[annotation]
    //  i.e. it automatically wraps the method in a try block and
    //  attaches a trace step to it
    res.map(|mesh_data| Mesh::from_data(ctx, mesh_data))
        .with_trace_step("grid_mesh")
}

pub fn grid_dot_mesh(gtx: &GameContext, ctx: &Context) -> Result<Mesh> {
    let CellDim { side, sin, cos } = gtx.cell_dim;
    let HexDim { h: board_h, v: board_v } = gtx.board_dim;

    let draw_mode = DrawMode::fill();
    let radius = gtx.palette.grid_dot_radius;
    let color = gtx.palette.grid_dot_color;

    let mut builder = MeshBuilder::new();
    let mut circle = |point| builder.circle(draw_mode, point, radius, 0.1, color).map(|_| {});

    let res: Result<_> = try {
        for h in 0..(board_h + 1) / 2 {
            let dh = h as f32 * 2. * (side + cos);

            for v in 0..=board_v {
                let dv = v as f32 * 2. * sin;

                circle(Point { x: cos + dh, y: dv })?;
                circle(Point { x: cos + side + dh, y: dv })?;

                // line between b and a
                if !(board_h.is_odd() && h == (board_h + 1) / 2 - 1) {
                    circle(Point {
                        x: 2. * cos + side + dh,
                        y: sin + dv,
                    })?;
                    circle(Point {
                        x: 2. * cos + 2. * side + dh,
                        y: sin + dv,
                    })?;
                }
            }
        }
        builder.build()
    };
    res.map(|mesh_data| Mesh::from_data(ctx, mesh_data))
        .with_trace_step("grid_mesh")
}

pub fn border_mesh(gtx: &GameContext, ctx: &Context) -> Result<Mesh> {
    let CellDim { side, sin, cos } = gtx.cell_dim;
    let HexDim { h: board_h, v: board_v } = gtx.board_dim;

    // two kinds of alternating vertical lines
    let mut vline_a = vec![]; // lines that start from the top with /
    let mut vline_b = vec![]; // lines that start from the top with \

    #[rustfmt::skip]
    for dv in (0..=board_v).map(|v| v as f32 * 2. * sin) {
        vline_a.push(Point { x: cos, y: dv });
        vline_a.push(Point { x: 0., y: dv + sin });
        vline_b.push(Point { x: cos + side, y: dv });
        vline_b.push(Point { x: 2. * cos + side, y: dv + sin });
    }

    let mut builder = MeshBuilder::new();

    let draw_mode = DrawMode::stroke(gtx.palette.border_thickness);
    let color = gtx.palette.border_color;

    let res: Result<_> = try {
        // left border
        builder.polyline(draw_mode, &vline_a[..vline_a.len() - 1], color)?;

        // right border
        let single_offset = 2. * (side + cos);
        if board_h.is_even() {
            let offset = (board_h / 2) as f32 * single_offset;
            vline_a.iter_mut().for_each(|a| a.x += offset);
            builder.polyline(draw_mode, &vline_a[1..], color)?;
        } else {
            let offset = ((board_h - 1) / 2) as f32 * single_offset;
            vline_b.iter_mut().for_each(|b| b.x += offset);
            builder.polyline(draw_mode, &vline_b[..vline_b.len() - 1], color)?;
        }

        let mut hline = vec![];
        for h in 0..board_h / 2 {
            let dh = 2. * (side + cos) * h as f32;
            hline.push(Point { x: dh + cos, y: 0. });
            hline.push(Point { x: dh + side + cos, y: 0. });
            hline.push(Point { x: dh + side + 2. * cos, y: sin });
            hline.push(Point {
                x: dh + 2. * side + 2. * cos,
                y: sin,
            });
        }
        if board_h.is_odd() {
            let dh = 2. * (side + cos) * (board_h / 2) as f32;
            hline.push(Point { x: dh + cos, y: 0. });
            hline.push(Point { x: dh + side + cos, y: 0. });
        }

        // top border
        builder.polyline(draw_mode, &hline, color)?;

        // bottom border
        // shift hline
        let offset = board_v as f32 * 2. * sin;
        hline.iter_mut().for_each(|p| p.y += offset);
        builder.polyline(draw_mode, &hline, color)?;

        builder.build()
    };

    res.map(|mesh_data| Mesh::from_data(ctx, mesh_data))
        .map_err(Error::from)
        .with_trace_step("border_mesh")
}
