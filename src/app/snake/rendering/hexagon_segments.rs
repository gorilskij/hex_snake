use crate::{
    app::snake::rendering::{
        descriptions::SegmentDescription,
        point_factory::SegmentRenderer,
        render_hexagon,
    },
    basic::{transformations::translate, Point},
};

pub struct HexagonSegments;

/// HexagonSegments makes no differentiation between straight, blunt, and sharp
/// segments, it does not implement the three specific functions (as these
/// shouldn't be used directly outside of `render_segment` and instead implements
/// `render_segment` directly
impl SegmentRenderer for HexagonSegments {
    fn render_straight_segment(_: &SegmentDescription) -> Vec<Point> {
        unreachable!()
    }
    fn render_curved_segment(_: &SegmentDescription, _turn: f32) -> Vec<Point> {
        unreachable!()
    }

    fn render_segment(description: SegmentDescription, _turn: f32) -> Vec<Point> {
        let mut points = render_hexagon(description.cell_dim);
        translate(&mut points, description.destination);
        points
    }
}
