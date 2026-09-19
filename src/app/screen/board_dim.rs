use macroquad::window::{screen_height, screen_width};

use crate::basic::{CellDim, HexDim, Point};

pub fn calculate_offset(board_dim: HexDim, cell_dim: CellDim) -> Point {
    let window_dim = Point {
        x: screen_width(),
        y: screen_height(),
    };
    let CellDim { sin, cos, .. } = cell_dim;

    let board_cartesian_dim = Point {
        x: board_dim.h as f32 * (cell_dim.width() - cos) + cos,
        y: board_dim.v as f32 * cell_dim.height() + sin,
    };
    (window_dim - board_cartesian_dim) / 2.
}

pub fn calculate_board_dim(cell_dim: CellDim) -> HexDim {
    let window_dim = Point {
        x: screen_width(),
        y: screen_height(),
    };
    let CellDim { sin, cos, .. } = cell_dim;

    HexDim {
        h: ((window_dim.x - cos) / (cell_dim.width() - cos)) as isize,
        v: ((window_dim.y - sin) / cell_dim.height()) as isize,
    }
}
