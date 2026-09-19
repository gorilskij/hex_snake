use crate::basic::Point;

/// Whether `point` lies inside the polygon `points` (even-odd rule).
///
/// Casts a ray straight down from the point and counts the edges it crosses:
/// an odd count means inside. An edge counts only if its endpoints lie
/// strictly on opposite sides of the ray (one `>`, the other `<=`), so a ray
/// through a vertex counts it once, and edges parallel to the ray are skipped.
pub fn shape_point(points: &[Point], point: Point) -> bool {
    let mut inside = false;
    for (&a, &b) in points.iter().zip(points.iter().skip(1).chain(points.first())) {
        if (a.x > point.x) != (b.x > point.x) {
            // y where the edge crosses the vertical line through the point;
            // a.x != b.x is guaranteed by the check above
            let y = a.y + (point.x - a.x) * (b.y - a.y) / (b.x - a.x);
            if y > point.y {
                inside = !inside;
            }
        }
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic::CellDim;
    use crate::rendering::shape::{Hexagon, Shape};

    #[test]
    fn hexagon_inside_and_outside() {
        let cell_dim = CellDim::from(10.);
        let hexagon = Hexagon::new(cell_dim);
        let (w, h) = (cell_dim.width(), cell_dim.height());

        assert!(shape_point(&hexagon, cell_dim.center()));
        assert!(!shape_point(&hexagon, Point { x: -1., y: h / 2. }));
        assert!(!shape_point(&hexagon, Point { x: w + 1., y: h / 2. }));
        // the corner regions of the bounding box are outside the hexagon
        assert!(!shape_point(&hexagon, Point { x: 0.5, y: 0.5 }));
    }

    #[test]
    fn ray_through_a_vertex_counts_it_once() {
        let cell_dim = CellDim::from(10.);
        let hexagon = Hexagon::new(cell_dim);
        // straight below this point lies the hexagon's bottom-left vertex
        let point = Point { x: cell_dim.cos, y: cell_dim.height() / 2. };
        assert!(shape_point(&hexagon, point));
    }
}
