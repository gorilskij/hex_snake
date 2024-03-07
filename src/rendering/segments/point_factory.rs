use ggez::graphics::{DrawMode, MeshBuilder};

use crate::error::{Error, ErrorConversion, Result};
use crate::rendering;
use crate::rendering::segments::descriptions::{Polygon, RoundHeadDescription, SegmentDescription};
use crate::rendering::segments::hexagon_segments::HexagonSegments;
use crate::rendering::segments::smooth_segments::SmoothSegments;

impl SegmentDescription {
    /// Render the segment into a list of drawable subsegments
    /// each represented as a list of points and a color,
    /// `snake_len` is used to calculate how many subsegments
    /// there should be (longer snakes have lower subsegment
    /// resolution)
    pub fn render(&self, color_resolution: usize, turn_fraction: f32) -> Box<dyn Iterator<Item = Polygon> + '_> {
        // TODO: pass prefs or some fragment of it instead of random arguments
        match self.draw_style {
            rendering::Style::Hexagon => HexagonSegments::render_segment(self, 0.0, RoundHeadDescription::Gone, 0),

            rendering::Style::Smooth => {
                let round_head = self.fraction.round_head_description(self.prev_fraction, self.cell_dim);

                SmoothSegments::render_segment(self, turn_fraction, round_head, color_resolution)
            }
        }
    }

    /// Returns number of polygons built
    pub fn build(self, builder: &mut MeshBuilder, color_resolution: usize) -> Result<usize> {
        let mut polygons = 0;
        let turn_fraction = self.turn.fraction;
        self.render(color_resolution, turn_fraction)
            .try_for_each(|Polygon { points, color }| {
                if points.len() >= 3 {
                    polygons += 1;
                    builder.polygon(DrawMode::fill(), &points, *color).map(|_| ())
                } else {
                    // TODO: re-enable (and switch to log levels)
                    // eprintln!("warning: SegmentDescription::render returned a Vec with < 3 points");
                    Ok(())
                }
            })
            .map_err(Error::from)
            .with_trace_step("SegmentDescription::build")?;
        Ok(polygons)
    }
}

// TODO: just have render_segment, the straight/curved distinction can be made by smooth_segments internally
// TODO: rework documentation (switched to subsegments)
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
        turn_fraction: f32,
        round_head: RoundHeadDescription,
        color_resolution: usize,
    ) -> Box<dyn Iterator<Item = Polygon> + '_>;
}
