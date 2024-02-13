use crate::basic::{CellDim, Point};
use crate::rendering::shape::Shape;

pub struct Hexagon;

impl Shape for Hexagon {
    fn raw_points(CellDim { side, cos, sin }: CellDim) -> Vec<Point> {
        vec![
            Point { x: cos, y: 0. },
            Point { x: cos + side, y: 0. },
            Point { x: cos * 2. + side, y: sin },
            Point { x: cos + side, y: sin * 2. },
            Point { x: cos, y: sin * 2. },
            Point { x: 0., y: sin },
        ]
    }
}
