use ggez::{mint::Point2, winit::dpi::PhysicalSize};
use std::ops::{Div, Mul};
use lyon_geom::euclid::default::{Point2D, Vector2D};
use std::marker::PhantomData;

/// A more convenient version of mint::Point2<f32>
#[derive(Copy, Clone, Debug, Add, AddAssign, Sub, SubAssign)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl From<Point> for Point2<f32> {
    fn from(Point { x, y }: Point) -> Self {
        Point2 { x, y }
    }
}

impl From<Point2<f32>> for Point {
    fn from(Point2 { x, y }: Point2<f32>) -> Self {
        Self { x, y }
    }
}

impl From<PhysicalSize<u32>> for Point {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self {
            x: size.width as f32,
            y: size.height as f32,
        }
    }
}

impl From<Point2D<f32>> for Point {
    fn from(Point2D { x, y, _unit }: Point2D<f32>) -> Self {
        Self { x, y }
    }
}

impl From<Point> for Point2D<f32> {
    fn from(Point { x, y }: Point) -> Self {
        Point2D { x, y, _unit: PhantomData }
    }
}

impl From<Point> for Vector2D<f32> {
    fn from(Point { x, y }: Point) -> Self {
        Vector2D { x, y, _unit: PhantomData }
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Mul<Point> for f32 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Point {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self { x: self.x / rhs, y: self.y / rhs }
    }
}

impl Point {
    /// Equal x and y
    pub fn square(side: f32) -> Self {
        Self { x: side, y: side }
    }

    #[must_use]
    pub fn rotate_clockwise(mut self, origin: Self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        self -= origin;
        self = Point {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        };
        self + origin
    }

    #[must_use]
    pub fn rotate_counterclockwise(self, origin: Self, angle: f32) -> Self {
        self.rotate_clockwise(origin, -angle)
    }

    #[must_use]
    pub fn magnitude(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
