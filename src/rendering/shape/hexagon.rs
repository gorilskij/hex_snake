use crate::basic::{CellDim, Point};
use crate::rendering::shape::{Shape, ShapePoints};

pub struct Hexagon;

impl Shape for Hexagon {
    fn points(CellDim { side, cos, sin }: CellDim) -> ShapePoints {
        vec![
            Point { x: cos, y: 0. },
            Point { x: cos + side, y: 0. },
            Point { x: cos * 2. + side, y: sin },
            Point { x: cos + side, y: sin * 2. },
            Point { x: cos, y: sin * 2. },
            Point { x: 0., y: sin },
        ]
        .into()
    }

    fn bounding_box(CellDim { side, cos, sin }: CellDim) -> Point {
        Point { x: cos * 2. + side, y: sin * 2. }
    }

    fn center(CellDim { side, cos, sin }: CellDim) -> Point {
        Point { x: cos + side / 2., y: sin }
    }
}
