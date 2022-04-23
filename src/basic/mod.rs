pub use cell_dim::CellDim;
pub use dir::Dir;
pub use dir12::Dir12;
pub use hex_point::{HexDim, HexPoint};
pub use point::Point;
use std::f32::consts::TAU;

mod cell_dim;
mod dir;
mod dir12;
mod hex_point;
mod point;
pub mod transformations;

macro_rules! hash_map {
    { $($key:expr => $value:expr),* $(,)? } => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key, $value); )*
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

/// Absolute difference between two angles (in radians)
pub fn angle_distance(angle1: f32, angle2: f32) -> f32 {
    // let d1 = (a1 - a2).abs();
    // // add tau to the smaller of the two angles and consider that distance as well
    // let b1 = partial_min(a1, a2).unwrap() + TAU;
    // let b2 = partial_max(a1, a2).unwrap();
    // let d2 = (b1 - b2).abs();
    // partial_min(d1, d2).unwrap()
    let dist = (angle1 - angle2).abs();
    // if the distance is more than halfway around the circle,
    // go around the other way
    if dist > TAU / 2. {
        TAU - dist
    } else {
        dist
    }
}
