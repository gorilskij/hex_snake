use ggez::graphics::{DrawMode, MeshBuilder};

use crate::error::{Error, ErrorConversion, Result};
use crate::rendering;
use crate::rendering::segments::descriptions::{Polygon, SegmentDescription};
use crate::rendering::segments::hexagon_segments::HexagonSegments;
use crate::rendering::segments::smooth_segments::SmoothSegments;

pub type ColorResolution = u16;

impl SegmentDescription {
    /// Render the segment into a list of drawable subsegments
    /// each represented as a list of points and a color,
    /// `snake_len` is used to calculate how many subsegments
    /// there should be (longer snakes have lower subsegment
    /// resolution)
    pub fn render(&self, color_resolution: ColorResolution) -> Box<dyn Iterator<Item = Polygon> + '_> {
        match self.draw_style {
            rendering::Style::Hexagon => HexagonSegments::render_segment(self, 0),
            rendering::Style::Smooth => SmoothSegments::render_segment(self, color_resolution),
        }
    }

    /// Return number of polygons built
    pub fn build(self, builder: &mut MeshBuilder, color_resolution: ColorResolution) -> Result<usize> {
        self.render(color_resolution)
            .try_fold(0, |polygons, Polygon { points, color, .. }| {
                builder.polygon(DrawMode::fill(), &points, *color).map(|_| polygons + 1)
            })
            .map_err(Error::from)
            .with_trace_step("SegmentDescription::build")
    }
}

// TODO: redo documentation
/// The `render_default_*` functions are without position or rotation,
/// they assume a default orientation and the transformation is performed
/// afterwards
pub trait SegmentRenderer {
    // /// Render a straight segment in the default orientation,
    // /// coming from above (U) and going down (D)
    // fn render_default_straight_segment(
    //     description: &SegmentDescription,
    //     fraction: SegmentFraction,
    //     round_head: RoundHeadDescription,
    // ) -> Vec<Point>;
    //
    // /// Render a curved segment in the default orientation,
    // /// a blunt segment coming from above (U) and going down-right (Dr)
    // /// or a sharp segment coming from above (U) and going up-right (Ur)
    // ///
    // /// `turn` describes how far along the segment is on its turn,
    // /// a value of 0 means the segment is straight, a value of 1 means
    // /// the turn is complete
    // fn render_default_curved_segment(
    //     description: &SegmentDescription,
    //     turn_fraction: f32,
    //     fraction: SegmentFraction,
    //     round_head: RoundHeadDescription,
    // ) -> Vec<Point>;

    /// Render a segment, rotate it and reflect it to match the desired
    /// coming-from and going-to directions, and translate it to match
    /// the desired position
    fn render_segment(
        description: &SegmentDescription,
        color_resolution: ColorResolution,
    ) -> Box<dyn Iterator<Item = Polygon> + '_>;
}
