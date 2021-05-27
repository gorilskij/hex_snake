use crate::{
    app::snake::rendering::{
        descriptions::{SegmentDescription, SegmentFraction},
        point_factory::SegmentRenderer,
        render_hexagon,
    },
    basic::{transformations::translate, CellDim, Point},
};

pub struct HexagonSegments;

/// HexagonSegments makes no differentiation between straight, blunt, and sharp
/// segments, it does not implement the three specific functions (as these
/// shouldn't be used directly outside of `render_segment` and instead implements
/// `render_segment_directly`
impl SegmentRenderer for HexagonSegments {
    fn render_default_straight(_: CellDim, _: SegmentFraction) -> Vec<Point> {
        unreachable!()
    }
    fn render_default_blunt(_: CellDim, _: SegmentFraction) -> Vec<Point> {
        unreachable!()
    }
    fn render_default_sharp(_: CellDim, _: SegmentFraction) -> Vec<Point> {
        unreachable!()
    }

    fn render_segment(description: SegmentDescription) -> Vec<Point> {
        let mut points = render_hexagon(description.cell_dim);
        translate(&mut points, description.destination);
        points
    }
}
