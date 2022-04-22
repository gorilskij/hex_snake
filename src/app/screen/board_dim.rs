use crate::basic::{CellDim, HexDim, Point};
use ggez::Context;

pub fn calculate_offset(ctx: &Context, board_dim: HexDim, cell_dim: CellDim) -> Point {
    let window_dim: Point = ggez::graphics::window(ctx).inner_size().into();
    let CellDim { side, sin, cos } = cell_dim;

    let board_cartesian_dim = Point {
        x: board_dim.h as f32 * (side + cos) + cos,
        y: board_dim.v as f32 * 2. * sin + sin,
    };
    (window_dim - board_cartesian_dim) / 2.
}

pub fn calculate_board_dim(ctx: &Context, cell_dim: CellDim) -> HexDim {
    let window_dim: Point = ggez::graphics::window(ctx).inner_size().into();
    let CellDim { side, sin, cos } = cell_dim;

    HexDim {
        h: ((window_dim.x - cos) / (side + cos)) as isize,
        v: ((window_dim.y - sin) / (2. * sin)) as isize,
    }
}
