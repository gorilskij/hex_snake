use crate::rendering::segments::descriptions::{Polygon, RoundHeadDescription, SegmentDescription};
use crate::rendering::segments::point_factory::SegmentRenderer;
use crate::rendering::shape::{Hexagon, Shape};
use std::iter;

pub struct HexagonSegments;

/// HexagonSegments makes no differentiation between straight, blunt, and sharp
/// segments, it does not implement the three specific functions (as these
/// shouldn't be used directly outside of `render_segment` and instead implements
/// `render_segment` directly
impl SegmentRenderer for HexagonSegments {
    fn render_segment(
        description: &SegmentDescription,
        _: f32,
        _: RoundHeadDescription,
        _: usize,
    ) -> Box<dyn Iterator<Item = Polygon> + '_> {
        let points = Hexagon::points(description.cell_dim)
            .translate(description.destination)
            .into();
        let poylgon = Polygon {
            points,
            color: description.segment_style.first_color(),
        };
        Box::new(iter::once(poylgon))
    }
}
