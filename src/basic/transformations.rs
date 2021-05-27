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
