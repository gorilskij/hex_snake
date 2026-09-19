use crate::basic::{CellDim, Point};
use crate::rendering::shape::Shape;

pub struct Hexagon;

impl Shape for Hexagon {
    fn raw_points(cell_dim: CellDim) -> Vec<Point> {
        let CellDim { side, cos, .. } = cell_dim;
        let (w, h) = (cell_dim.width(), cell_dim.height());
        vec![
            Point { x: cos, y: 0. },
            Point { x: cos + side, y: 0. },
            Point { x: w, y: h / 2. },
            Point { x: cos + side, y: h },
            Point { x: cos, y: h },
            Point { x: 0., y: h / 2. },
        ]
    }
}
