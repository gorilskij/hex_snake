use std::ops::Deref;

pub use arrow::ArrowLeft;
pub use hexagon::Hexagon;
pub use triangle_arrow::TriangleArrowLeft;
pub use wide_hexagon::WideHexagon;

use crate::basic::{CellDim, Point};
use crate::support::partial_min_max::PartialMinMax;

mod arrow;
pub mod collisions;
mod hexagon;
mod triangle_arrow;
mod wide_hexagon;

struct Line {
    start: Point,
    end: Point,
}

fn bounding_box_of(points: &[Point]) -> (Point, Point) {
    let (min_x, max_x) = points.iter().map(|p| p.x).partial_minmax_copy().unwrap_or((0., 0.));
    let (min_y, max_y) = points.iter().map(|p| p.y).partial_minmax_copy().unwrap_or((0., 0.));
    (Point { x: min_x, y: min_y }, Point { x: max_x, y: max_y })
}

fn center_of(points: &[Point]) -> Point {
    let (a, b) = bounding_box_of(points);
    (a + b) / 2.
}

pub trait Shape {
    fn raw_points(cell_dim: CellDim) -> Vec<Point>;

    fn center(cell_dim: CellDim) -> Point {
        center_of(&Self::raw_points(cell_dim))
    }

    fn new(cell_dim: CellDim) -> ShapePoints {
        ShapePoints {
            points: Self::raw_points(cell_dim),
            center: Self::center(cell_dim),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShapePoints {
    points: Vec<Point>,
    center: Point,
}

impl From<Vec<Point>> for ShapePoints {
    fn from(points: Vec<Point>) -> Self {
        Self { center: center_of(&points), points }
    }
}

impl From<ShapePoints> for Vec<Point> {
    fn from(value: ShapePoints) -> Self {
        value.points
    }
}

impl ShapePoints {
    pub fn points(&self) -> &ShapePointsSlice {
        unsafe { std::mem::transmute(self.points.as_slice()) }
    }

    pub fn bounding_box(&self) -> (Point, Point) {
        bounding_box_of(self.points())
    }

    pub fn center(&self) -> Point {
        self.center
    }

    pub fn rotate_clockwise(mut self, origin: Point, angle: f32) -> Self {
        self.points
            .iter_mut()
            .for_each(|point| *point = point.rotate_clockwise(origin, angle));
        self.center = self.center.rotate_clockwise(origin, angle);
        self
    }

    pub fn rotate_clockwise_about_center(mut self, angle: f32) -> Self {
        self.points
            .iter_mut()
            .for_each(|point| *point = point.rotate_clockwise(self.center, angle));
        self
    }

    pub fn translate(mut self, delta: Point) -> Self {
        self.points.iter_mut().for_each(|point| *point += delta);
        self.center += delta;
        self
    }

    /// Mirrors the image in the y axis
    pub fn flip_horizontally(mut self, x: f32) -> Self {
        self.points.iter_mut().for_each(|point| point.x = 2. * x - point.x);
        self.center.x = 2. * x - self.center.x;
        self
    }
}

#[derive(Debug)]
pub struct ShapePointsSlice([Point]);

impl Deref for ShapePoints {
    type Target = ShapePointsSlice;

    fn deref(&self) -> &Self::Target {
        self.points()
    }
}

impl Deref for ShapePointsSlice {
    type Target = [Point];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
