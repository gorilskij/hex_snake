use crate::basic::*;

pub mod descriptions;
mod hexagon_segments;
pub mod point_factory;
mod smooth_segments;

pub fn render_hexagon(CellDim { side, sin, cos }: CellDim) -> Vec<Point> {
    vec![
        Point { x: cos, y: 0. },
        Point { x: cos + side, y: 0. },
        Point { x: cos * 2. + side, y: sin },
        Point { x: cos + side, y: sin * 2. },
        Point { x: cos, y: sin * 2. },
        Point { x: 0., y: sin },
    ]
}
