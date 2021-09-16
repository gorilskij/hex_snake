use crate::basic::Point;
use std::ops::Mul;

#[derive(Copy, Clone, Mul)]
pub struct CellDim {
    pub side: f32,
    // sin is longer than cos
    // they describe the height and width of the diagonal segments of
    // a hexagon with its flat segments horizontal on the top and bottom
    pub sin: f32,
    pub cos: f32,
}

impl From<f32> for CellDim {
    fn from(side: f32) -> Self {
        use std::f32::consts::FRAC_PI_3;
        Self {
            side,
            sin: FRAC_PI_3.sin() * side,
            cos: FRAC_PI_3.cos() * side,
        }
    }
}

impl CellDim {
    pub fn center(self) -> Point {
        Point {
            x: self.cos + self.side / 2.,
            y: self.sin,
        }
    }
}
