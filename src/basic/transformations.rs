use crate::basic::Point;

pub fn rotate_clockwise(points: &mut [Point], origin: Point, angle: f32) {
    points
        .iter_mut()
        .for_each(|point| *point = point.rotate_clockwise(origin, angle))
}

pub fn translate(points: &mut [Point], delta: Point) {
    points.iter_mut().for_each(|point| *point += delta)
}

/// Mirrors the image in the y axis
pub fn flip_horizontally(points: &mut [Point], x: f32) {
    points
        .iter_mut()
        .for_each(|point| point.x = 2. * x - point.x)
}

pub struct RotateClockwise<I> {
    iter: I,
    origin: Point,
    angle: f32,
}

impl<I: Iterator<Item = Vec<Point>>> Iterator for RotateClockwise<I> {
    type Item = Vec<Point>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut points = self.iter.next()?;
        rotate_clockwise(&mut points, self.origin, self.angle);
        Some(points)
    }
}

pub struct Translate<I> {
    iter: I,
    delta: Point,
}

impl<I: Iterator<Item = Vec<Point>>> Iterator for Translate<I> {
    type Item = Vec<Point>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut points = self.iter.next()?;
        translate(&mut points, self.delta);
        Some(points)
    }
}

pub struct FlipHorizontally<I> {
    iter: I,
    x: f32,
}

impl<I: Iterator<Item = Vec<Point>>> Iterator for FlipHorizontally<I> {
    type Item = Vec<Point>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut points = self.iter.next()?;
        flip_horizontally(&mut points, self.x);
        Some(points)
    }
}

pub trait IntoTransformationIter: Sized {
    fn rotate_clockwise(self, origin: Point, angle: f32) -> RotateClockwise<Self>;
    fn translate(self, delta: Point) -> Translate<Self>;
    fn flip_horizontally(self, x: f32) -> FlipHorizontally<Self>;
}

impl<I: Iterator<Item = Vec<Point>>> IntoTransformationIter for I {
    fn rotate_clockwise(self, origin: Point, angle: f32) -> RotateClockwise<Self> {
        RotateClockwise { iter: self, origin, angle }
    }

    fn translate(self, delta: Point) -> Translate<Self> {
        Translate { iter: self, delta }
    }

    fn flip_horizontally(self, x: f32) -> FlipHorizontally<Self> {
        FlipHorizontally { iter: self, x }
    }
}
