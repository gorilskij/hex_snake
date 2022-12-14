use crate::basic::transformations::translate;
use crate::basic::Point;
use crate::rendering::segments::descriptions::SegmentDescription;
use crate::rendering::segments::point_factory::SegmentRenderer;
use crate::rendering::segments::render_hexagon;

pub struct HexagonSegments;

/// HexagonSegments makes no differentiation between straight, blunt, and sharp
/// segments, it does not implement the three specific functions (as these
/// shouldn't be used directly outside of `render_segment` and instead implements
/// `render_segment` directly
impl SegmentRenderer for HexagonSegments {
    fn render_default_straight_segment(_: &SegmentDescription) -> Vec<Point> {
        unreachable!()
    }
    fn render_default_curved_segment(_: &SegmentDescription, _turn_fraction: f32) -> Vec<Point> {
        unreachable!()
    }

    fn render_segment(description: SegmentDescription, _turn_fraction: f32) -> Vec<Point> {
        let mut points = render_hexagon(description.cell_dim);
        translate(&mut points, description.destination);
        points
    }
}
