pub use arrow::ArrowLeft;
pub use triangle_arrow::TriangleArrowLeft;

use crate::basic::{CellDim, Point};
use crate::support::partial_min_max::PartialMinMax;

mod arrow;
pub mod collisions;
mod triangle_arrow;

struct Line {
    start: Point,
    end: Point,
}

pub trait Shape {
    fn points(cell_dim: CellDim) -> Vec<Point>;

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
