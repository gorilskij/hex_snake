use crate::basic::{CellDim, Point};
use crate::rendering::shape::{Shape, ShapePoints};

/// A hexagon stretched horizontally: its top and bottom sides are longer
/// than the diagonal ones.
pub struct WideHexagon;

impl WideHexagon {
    const H_SIDE_MULTIPLIER: f32 = 5.;

    /// A wide hexagon whose top and bottom sides are `h_side` long.
    pub fn with_h_side(cell_dim: CellDim, h_side: f32) -> ShapePoints {
        Self::points(cell_dim, h_side).into()
    }

    fn points(cell_dim: CellDim, h_side: f32) -> Vec<Point> {
        let CellDim { cos, .. } = cell_dim;
        let h = cell_dim.height();
        vec![
            Point { x: cos, y: 0. },
            Point { x: cos + h_side, y: 0. },
            Point { x: 2. * cos + h_side, y: h / 2. },
            Point { x: cos + h_side, y: h },
            Point { x: cos, y: h },
            Point { x: 0., y: h / 2. },
        ]
    }
}

impl Shape for WideHexagon {
    fn raw_points(cell_dim: CellDim) -> Vec<Point> {
        Self::points(cell_dim, Self::H_SIDE_MULTIPLIER * cell_dim.side)
    }
}
