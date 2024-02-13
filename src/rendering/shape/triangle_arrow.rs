use crate::basic::{CellDim, Point};
use crate::rendering::shape::{Shape, ShapePoints};
use std::f32::consts::TAU;

pub struct TriangleArrowLeft;

impl Shape for TriangleArrowLeft {
    fn points(CellDim { side, .. }: CellDim) -> ShapePoints {
        let width = side / 2. * (TAU / 6.).tan();
        vec![
            Point { x: 0., y: side / 2. },
            Point { x: width, y: side },
            Point { x: width, y: 0. },
        ]
        .into()
    }

    fn bounding_box(CellDim { side, .. }: CellDim) -> Point {
        Point {
            x: side / 2. * (TAU / 6.).tan(),
            y: side,
        }
    }

    // not the center of the bounding box!
    fn center(CellDim { side, .. }: CellDim) -> Point {
        let width = side / 2. * (TAU / 6.).tan();
        Point { x: 2. / 3. * width, y: side / 2. }
    }
}
