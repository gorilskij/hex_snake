use crate::basic::Point;

#[derive(Copy, Clone, Mul, Debug)]
pub struct CellDim {
    pub side: f32,
    // sin is longer than cos
    // they describe the height and width of the diagonal segments of
    // a hexagon with its flat segments horizontal on the top and bottom
    pub sin: f32,
    pub cos: f32,
}

impl Default for CellDim {
    fn default() -> Self {
        Self::from(30.)
    }
}

impl From<f32> for CellDim {
    fn from(side: f32) -> Self {
        use std::f32::consts::FRAC_PI_3;
        Self {
            side,
            sin: FRAC_PI_3.sin() * side,
            cos: 0.5 * side,
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

impl PartialEq for CellDim {
    fn eq(&self, other: &Self) -> bool {
        (self.side - other.side).abs() < f32::EPSILON
    }
}

impl Eq for CellDim {}
