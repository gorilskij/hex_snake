use num_integer::Integer;

use crate::app::game_context::GameContext;
use crate::basic::{CellDim, HexDim, Point};
use anyhow::Result;
use crate::support::mesh::{build_circle, build_line, build_polyline, DrawMode, Mesh};

// TODO: make this readable
// TODO: add option to exclude border from grid mesh
//  when border is drawn separately
pub fn grid_mesh(gtx: &GameContext) -> Result<Mesh> {
    let CellDim { side, sin, cos } = gtx.cell_dim;
    let HexDim { h: board_h, v: board_v } = gtx.board_dim;

    // two kinds of alternating vertical lines
    let cap = 2 * (board_v as usize + 1);
    let mut vline_a = Vec::with_capacity(cap); // lines that start from the top with /
    let mut vline_b = Vec::with_capacity(cap); // lines that start from the top with \

    #[rustfmt::skip]
    for dv in (0..=board_v).map(|v| v as f32 * 2. * sin) {
        vline_a.push(Point { x: cos, y: dv });
        vline_a.push(Point { x: 0., y: dv + sin });
        vline_b.push(Point { x: cos + side, y: dv });
        vline_b.push(Point { x: 2. * cos + side, y: dv + sin });
    }

    let mut parts: Vec<Mesh> = vec![];

    let draw_mode = DrawMode::stroke(gtx.palette.grid_thickness);
    let color = gtx.palette.grid_color;

    for h in 0..(board_h + 1) / 2 {
        if h == 0 {
            parts.push(build_polyline(draw_mode, &vline_a[..vline_a.len() - 1], color));
        } else {
            parts.push(build_polyline(draw_mode, &vline_a, color));
        }
        if board_h.is_odd() && h == (board_h + 1) / 2 - 1 {
            parts.push(build_polyline(draw_mode, &vline_b[..vline_b.len() - 1], color));
        } else {
            parts.push(build_polyline(draw_mode, &vline_b, color));
        }

        let dh = h as f32 * 2. * (side + cos);

        for v in 0..=board_v {
            let dv = v as f32 * 2. * sin;

            // line between a and b
            parts.push(build_line(
                #[rustfmt::skip] &[
                    Point { x: cos + dh, y: dv },
                    Point { x: cos + side + dh, y: dv },
                ],
                gtx.palette.grid_thickness,
                color,
            ));

            // line between b and a
            if !(board_h.is_odd() && h == (board_h + 1) / 2 - 1) {
                parts.push(build_line(
                    #[rustfmt::skip] &[
                        Point { x: 2. * cos + side + dh, y: sin + dv },
                        Point { x: 2. * cos + 2. * side + dh, y: sin + dv },
                    ],
                    gtx.palette.grid_thickness,
                    color,
                ));
            }
        }

        // shift the lines right by 2 cells
        let offset = 2. * (side + cos);
        vline_a.iter_mut().for_each(|a| a.x += offset);
        vline_b.iter_mut().for_each(|b| b.x += offset);
    }
    if board_h.is_even() {
        parts.push(build_polyline(draw_mode, &vline_a[1..], color));
    }

    Ok(Mesh::combine(parts))
}

pub fn grid_dot_mesh(gtx: &GameContext) -> Result<Mesh> {
    let CellDim { side, sin, cos } = gtx.cell_dim;
    let HexDim { h: board_h, v: board_v } = gtx.board_dim;

    let draw_mode = DrawMode::fill();
    let radius = gtx.palette.grid_dot_radius;
    let color = gtx.palette.grid_dot_color;

    let mut parts: Vec<Mesh> = vec![];
    let mut circle = |point| parts.push(build_circle(draw_mode, point, radius, color));

    for h in 0..(board_h + 1) / 2 {
        let dh = h as f32 * 2. * (side + cos);

        for v in 0..=board_v {
            let dv = v as f32 * 2. * sin;

            circle(Point { x: cos + dh, y: dv });
            circle(Point { x: cos + side + dh, y: dv });

            // line between b and a
            if !(board_h.is_odd() && h == (board_h + 1) / 2 - 1) {
                circle(Point {
                    x: 2. * cos + side + dh,
                    y: sin + dv,
                });
                circle(Point {
                    x: 2. * cos + 2. * side + dh,
                    y: sin + dv,
                });
            }
        }
    }
    Ok(Mesh::combine(parts))
}

pub fn border_mesh(gtx: &GameContext) -> Result<Mesh> {
    let CellDim { side, sin, cos } = gtx.cell_dim;
    let HexDim { h: board_h, v: board_v } = gtx.board_dim;

    // two kinds of alternating vertical lines
    let cap = 2 * (board_v as usize + 1);
    let mut vline_a = Vec::with_capacity(cap); // lines that start from the top with /
    let mut vline_b = Vec::with_capacity(cap); // lines that start from the top with \

    #[rustfmt::skip]
    for dv in (0..=board_v).map(|v| v as f32 * 2. * sin) {
        vline_a.push(Point { x: cos, y: dv });
        vline_a.push(Point { x: 0., y: dv + sin });
        vline_b.push(Point { x: cos + side, y: dv });
        vline_b.push(Point { x: 2. * cos + side, y: dv + sin });
    }

    let mut parts: Vec<Mesh> = vec![];

    let draw_mode = DrawMode::stroke(gtx.palette.border_thickness);
    let color = gtx.palette.border_color;

    // left border
    parts.push(build_polyline(draw_mode, &vline_a[..vline_a.len() - 1], color));

    // right border
    let single_offset = 2. * (side + cos);
    if board_h.is_even() {
        let offset = (board_h / 2) as f32 * single_offset;
        vline_a.iter_mut().for_each(|a| a.x += offset);
        parts.push(build_polyline(draw_mode, &vline_a[1..], color));
    } else {
        let offset = ((board_h - 1) / 2) as f32 * single_offset;
        vline_b.iter_mut().for_each(|b| b.x += offset);
        parts.push(build_polyline(draw_mode, &vline_b[..vline_b.len() - 1], color));
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
    parts.push(build_polyline(draw_mode, &hline, color));

    // bottom border
    // shift hline
    let offset = board_v as f32 * 2. * sin;
    hline.iter_mut().for_each(|p| p.y += offset);
    parts.push(build_polyline(draw_mode, &hline, color));

    Ok(Mesh::combine(parts))
}
