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

macro_rules! hash_map {
    { $($key:expr => $value:expr),* $(,)? } => {{
        let map = ::std::collections::HashMap::new();
        $( m.insert($key, $value); )*
        map
    }};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(dead_code)]
pub enum Side {
    Left,
    Right,
}

/// (graphics frame number, frame fraction)
pub type FrameStamp = (usize, f32);
