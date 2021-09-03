pub use cell_dim::CellDim;
pub use dir::Dir;
pub use dir12::Dir12;
pub use hex_point::{HexDim, HexPoint};
pub use point::Point;

mod cell_dim;
mod dir;
mod dir12;
mod hex_point;
mod point;
pub mod transformations;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DrawStyle {
    Hexagon,
    Smooth,
}
