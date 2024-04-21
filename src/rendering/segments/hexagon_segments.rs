use crate::rendering::point_factory::ColorResolution;
use crate::rendering::segments::descriptions::{Polygon, SegmentDescription};
use crate::rendering::segments::point_factory::SegmentRenderer;
use crate::rendering::shape::{Hexagon, Shape};

pub struct HexagonSegments;

/// HexagonSegments makes no differentiation between straight, blunt, and sharp segments
impl SegmentRenderer for HexagonSegments {
    fn render_segment(description: &SegmentDescription, _: ColorResolution) -> Box<dyn Iterator<Item = Polygon> + '_> {
        let points = Hexagon::new(description.cell_dim)
            .translate(description.destination)
            .into();
        let polygon = Polygon::new(
            // TODO: find a better way to transmit this that doesn't pollute the renderers which don't need it
            0,
            points,
            description.segment_style.first_color(),
        );
        Box::new(polygon.into_iter())
    }
}
