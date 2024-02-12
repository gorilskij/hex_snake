use crate::basic::{CellDim, Point};
use crate::rendering::shape::Shape;

pub struct ArrowLeft;

impl Shape for ArrowLeft {
    fn points(CellDim { side, sin, .. }: CellDim) -> Vec<Point> {
        vec![
            Point { x: 0., y: sin },
            Point { x: sin, y: 2. * sin },
            Point { x: sin, y: 1.5 * sin },
            Point { x: sin + side, y: 1.5 * sin },
            Point { x: sin + side, y: 0.5 * sin },
            Point { x: sin, y: 0.5 * sin },
            Point { x: sin, y: 0. },
        ]
    }
}
