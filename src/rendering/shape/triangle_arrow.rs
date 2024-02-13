use crate::basic::{CellDim, Point};
use crate::rendering::shape::Shape;
use std::f32::consts::TAU;

pub struct TriangleArrowLeft;

impl Shape for TriangleArrowLeft {
    fn points(CellDim { side, .. }: CellDim) -> Vec<Point> {
        let width = side / 2. * (TAU / 6.).tan();
        vec![
            Point { x: 0., y: side / 2. },
            Point { x: width, y: side },
            Point { x: width, y: 0. },
        ]
    }

    fn center(CellDim { side, .. }: CellDim) -> Point {
        let width = side / 2. * (TAU / 6.).tan();
        Point { x: 2. / 3. * width, y: side / 2. }
    }
}
