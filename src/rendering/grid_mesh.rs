use anyhow::Result;
use macroquad::color::Color;
use num_integer::Integer;

use crate::app::game_context::GameContext;
use crate::basic::{CellDim, Dir, HexDim, HexPoint, Point};
use crate::rendering::shape::{Hexagon, Shape};
use crate::support::mesh::{build_circle, build_line, DrawMode, Mesh};

/// The grid over the whole board, one line per cell edge.
pub fn grid_mesh(gtx: &GameContext) -> Result<Mesh> {
    let board_dim = gtx.board_dim;
    Ok(region_grid_mesh(
        board_cells(board_dim),
        |pos| board_dim.contains(pos),
        gtx.cell_dim,
        gtx.palette.grid_thickness,
        gtx.palette.grid_color,
    ))
}

/// The border around the whole board, one line per cell edge.
pub fn border_mesh(gtx: &GameContext) -> Result<Mesh> {
    let board_dim = gtx.board_dim;
    Ok(region_border_mesh(
        board_cells(board_dim),
        |pos| board_dim.contains(pos),
        gtx.cell_dim,
        gtx.palette.border_thickness,
        gtx.palette.border_color,
    ))
}

fn board_cells(board_dim: HexDim) -> impl Iterator<Item = HexPoint> {
    (0..board_dim.h).flat_map(move |h| (0..board_dim.v).map(move |v| HexPoint { h, v }))
}

/// The grid over any set of cells (`cells`, all of which satisfy
/// `contains`): every edge of every cell, each drawn once — an edge between
/// two cells of the set belongs to the cell above it (whose edge faces `U`,
/// `Ur` or `Dr`).
pub fn region_grid_mesh(
    cells: impl Iterator<Item = HexPoint>,
    contains: impl Fn(HexPoint) -> bool,
    cell_dim: CellDim,
    thickness: f32,
    color: Color,
) -> Mesh {
    edges_mesh(cells, cell_dim, thickness, color, |pos, dir| {
        matches!(dir, Dir::U | Dir::Ur | Dir::Dr) || !contains(pos.translate(dir, 1))
    })
}

/// The border around any set of cells (`cells`, all of which satisfy
/// `contains`): the edges between a cell of the set and one outside it.
pub fn region_border_mesh(
    cells: impl Iterator<Item = HexPoint>,
    contains: impl Fn(HexPoint) -> bool,
    cell_dim: CellDim,
    thickness: f32,
    color: Color,
) -> Mesh {
    edges_mesh(cells, cell_dim, thickness, color, |pos, dir| {
        !contains(pos.translate(dir, 1))
    })
}

/// A dot on every corner of `cells`, each drawn once.
pub fn region_dot_mesh(cells: impl Iterator<Item = HexPoint>, cell_dim: CellDim, radius: f32, color: Color) -> Mesh {
    // neighbouring cells share corners; compare them to a fraction of a pixel
    let mut seen = std::collections::HashSet::new();
    let parts = cells.flat_map(|pos| {
        let corners: Vec<Point> = Hexagon::new(cell_dim).translate(pos.to_cartesian(cell_dim)).into();
        corners
    });
    let parts = parts
        .filter(|p| seen.insert(((p.x * 16.).round() as i64, (p.y * 16.).round() as i64)))
        .map(|p| build_circle(DrawMode::fill(), p, radius, color))
        .collect::<Vec<_>>();
    Mesh::combine(parts)
}

/// One line for each edge of `cells` that `draw(cell, dir)` picks. Lines have
/// round ends, so edges meet cleanly at corners.
fn edges_mesh(
    cells: impl Iterator<Item = HexPoint>,
    cell_dim: CellDim,
    thickness: f32,
    color: Color,
    draw: impl Fn(HexPoint, Dir) -> bool,
) -> Mesh {
    let draw = &draw;
    let parts = cells.flat_map(|pos| {
        let corners: Vec<Point> = Hexagon::new(cell_dim).translate(pos.to_cartesian(cell_dim)).into();
        Dir::iter().filter(move |&dir| draw(pos, dir)).map(move |dir| {
            let (a, b) = edge(dir);
            build_line(&[corners[a], corners[b]], thickness, color)
        })
    });
    Mesh::combine(parts)
}

/// The corners (as indices into [`Hexagon`]'s points) of a cell's edge that
/// faces `dir`
fn edge(dir: Dir) -> (usize, usize) {
    match dir {
        Dir::U => (0, 1),
        Dir::Ur => (1, 2),
        Dir::Dr => (2, 3),
        Dir::D => (3, 4),
        Dir::Dl => (4, 5),
        Dir::Ul => (5, 0),
    }
}

pub fn grid_dot_mesh(gtx: &GameContext) -> Result<Mesh> {
    let CellDim { side, cos, .. } = gtx.cell_dim;
    let (width, height) = (gtx.cell_dim.width(), gtx.cell_dim.height());
    let HexDim { h: board_h, v: board_v } = gtx.board_dim;

    let draw_mode = DrawMode::fill();
    let radius = gtx.palette.grid_dot_radius;
    let color = gtx.palette.grid_dot_color;

    let mut parts: Vec<Mesh> = vec![];
    let mut circle = |point| parts.push(build_circle(draw_mode, point, radius, color));

    for h in 0..(board_h + 1) / 2 {
        let dh = h as f32 * 2. * (side + cos);

        for v in 0..=board_v {
            let dv = v as f32 * height;

            circle(Point { x: cos + dh, y: dv });
            circle(Point { x: cos + side + dh, y: dv });

            // line between b and a
            if !(board_h.is_odd() && h == (board_h + 1) / 2 - 1) {
                circle(Point { x: width + dh, y: height / 2. + dv });
                circle(Point {
                    x: width + side + dh,
                    y: height / 2. + dv,
                });
            }
        }
    }
    Ok(Mesh::combine(parts))
}
