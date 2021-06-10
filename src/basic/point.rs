use ggez::mint::Point2;
use std::ops::{Div, Mul};

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
        Point { x, y }
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
}
