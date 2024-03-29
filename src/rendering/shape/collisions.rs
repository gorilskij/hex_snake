use super::Line;
use crate::basic::Point;

// a b
// c d
const fn determinant_2x2(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * d - b * c
}

/// Project a line vertically from `vertical_line_start` to infinity.
/// Check whether this line intersects `bounded_line`.
fn vertical_downward_line_bounded_line(vertical_line_start: Point, bounded_line: Line) -> bool {
    let Point { x: x1, y: y1 } = vertical_line_start;
    let Point { x: x2, y: y2 } = vertical_line_start + Point { x: 0., y: 1. };
    let Point { x: x3, y: y3 } = bounded_line.start;
    let Point { x: x4, y: y4 } = bounded_line.end;

    #[rustfmt::skip]
    let u = determinant_2x2(
        x1 - x2, x1 - x3,
        y1 - y2, y1 - y3,
    ) / determinant_2x2(
        x1 - x2, x3 - x4,
        y1 - y2, y3 - y4,
    );

    if !(-1. ..=0.).contains(&u) {
        return false;
    }

    #[rustfmt::skip]
    let t = determinant_2x2(
        x1 - x3, x3 - x4,
        y1 - y3, y3 - y4,
    ) / determinant_2x2(
        x1 - x2, x3 - x4,
        y1 - y2, y3 - y4,
    );

    t >= 0.
}

/// Project a line straight down from the point, if this line intersects
/// the shape an odd number of times, then the point must be inside the shape
pub fn shape_point(points: &[Point], point: Point) -> bool {
    let num_intersections = points
        .iter()
        .zip(points.iter().skip(1).chain(points.first()))
        .filter(|win| {
            let (&start, &end) = win;
            let bounded_line = Line { start, end };
            vertical_downward_line_bounded_line(point, bounded_line)
        })
        .count();
    if num_intersections % 2 == 1 {
        println!("shape <> point: {num_intersections} intersections");
        true
    } else {
        false
    }
}
