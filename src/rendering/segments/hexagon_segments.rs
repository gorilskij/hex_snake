use crate::basic::transformations::translate;
use crate::basic::Point;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction};
use crate::rendering::segments::point_factory::SegmentRenderer;
use crate::rendering::segments::render_hexagon;

pub struct HexagonSegments;

/// HexagonSegments makes no differentiation between straight, blunt, and sharp
/// segments, it does not implement the three specific functions (as these
/// shouldn't be used directly outside of `render_segment` and instead implements
/// `render_segment` directly
impl SegmentRenderer for HexagonSegments {
    fn render_default_straight_segment(
        _: &SegmentDescription,
        _: SegmentFraction,
        _: Option<SegmentFraction>,
        _: Option<SegmentFraction>,
    ) -> Vec<Point> {
        unreachable!()
    }

    fn render_default_curved_segment(
        _: &SegmentDescription,
        _: f32,
        _: SegmentFraction,
    ) -> Vec<Point> {
        unreachable!()
    }

    fn render_segment(
        description: &SegmentDescription,
        _turn_fraction: f32,
        _: SegmentFraction,
        _: Option<SegmentFraction>,
        _: Option<SegmentFraction>,
    ) -> Vec<Point> {
        let mut points = render_hexagon(description.cell_dim);
        translate(&mut points, description.destination);
        points
    }
}
