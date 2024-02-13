pub use arrow::ArrowLeft;
pub use hexagon::Hexagon;
pub use triangle_arrow::TriangleArrowLeft;

use crate::basic::{CellDim, Point};
use crate::support::partial_min_max::PartialMinMax;

mod arrow;
pub mod collisions;
mod hexagon;
mod triangle_arrow;

struct Line {
    start: Point,
    end: Point,
}

pub trait Shape {
    fn points(cell_dim: CellDim) -> ShapePoints;

    fn bounding_box(cell_dim: CellDim) -> Point {
        let points = Self::points(cell_dim);
        Point {
            x: points.iter().map(|p| p.x).partial_max().unwrap_or(0.),
            y: points.iter().map(|p| p.y).partial_max().unwrap_or(0.),
        }
    }

    fn center(cell_dim: CellDim) -> Point {
        Self::bounding_box(cell_dim) / 2.
    }
}

#[derive(From, Into, Deref, DerefMut)]
pub struct ShapePoints(Vec<Point>);

impl ShapePoints {
    pub fn rotate_clockwise(mut self, origin: Point, angle: f32) -> Self {
        self.0
            .iter_mut()
            .for_each(|point| *point = point.rotate_clockwise(origin, angle));
        self
    }

    pub fn translate(mut self, delta: Point) -> Self {
        self.0.iter_mut().for_each(|point| *point += delta);
        self
    }

    /// Mirrors the image in the y axis
    pub fn flip_horizontally(mut self, x: f32) -> Self {
        self.0
            .iter_mut()
            .for_each(|point| point.x = 2. * x - point.x);
        self
    }
}
