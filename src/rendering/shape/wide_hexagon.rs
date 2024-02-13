use crate::basic::{CellDim, Point};
use crate::rendering::shape::Shape;

pub struct WideHexagon;

impl WideHexagon {
    const H_SIDE_MULTIPLIER: f32 = 5.;
}

impl Shape for WideHexagon {
    fn raw_points(CellDim { side, cos, sin }: CellDim) -> Vec<Point> {
        let h_side = Self::H_SIDE_MULTIPLIER * side;
        vec![
            Point { x: cos, y: 0. },
            Point { x: cos + h_side, y: 0. },
            Point { x: cos * 2. + h_side, y: sin },
            Point { x: cos + h_side, y: sin * 2. },
            Point { x: cos, y: sin * 2. },
            Point { x: 0., y: sin },
        ]
        .into()
    }
}
