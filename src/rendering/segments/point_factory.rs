use ggez::graphics::{DrawMode, MeshBuilder};
use rand::thread_rng;
use std::iter;

use crate::basic::Point;
use crate::color::Color;
use crate::error::{Error, ErrorConversion, Result};
use crate::rendering;
use crate::rendering::segments::descriptions::{
    RoundHeadDescription, SegmentDescription, SegmentFraction,
};
use crate::rendering::segments::hexagon_segments::HexagonSegments;
use crate::rendering::segments::smooth_segments::SmoothSegments;

struct Subsegment {
    subsegment_idx: usize,
    color: Color,
    // start assumed to be the end of the previous subsegment
    // or the start of the parent segment
    end: f32,
}

impl SegmentDescription {
    /// Split a single segment description into `n` subsegments,
    /// this is used to assign a solid color to each subsegment and thus
    /// simulate a smooth gradient
    fn get_subsegments(&self, num_subsegments: usize) -> impl Iterator<Item = Subsegment> + '_ {
        if self.draw_style == rendering::Style::Hexagon {
            unreachable!("hexagon segments don't support gradients")
        }

        let SegmentFraction { start, end } = self.fraction;
        let segment_size = self.fraction.end - self.fraction.start;

        let get_color = self.segment_style.color_at_fraction();

        let start_subsegment = (num_subsegments as f32 * start) as usize;
        let end_subsegment = (num_subsegments as f32 * end).ceil() as usize;
        // the actual number of subsegments (partial segments will
        //  have fewer than expected)
        let real_num_subsegments = (start_subsegment..end_subsegment).len();
        let subsegment_size = segment_size / real_num_subsegments as f32;

        // TODO: make sure we're not generating duplicate colors (that num_subsegments isn't too high)
        // the order is tail to head (opposite to the order in which snake segments are rendered)
        (start_subsegment..end_subsegment)
            // TODO: this is very awkward, remove double enumerate
            .rev()
            .enumerate()
            .rev()
            .map(move |(subsegment_idx, subsegment)| {
                (
                    subsegment_idx,
                    get_color(subsegment as f64 / num_subsegments as f64),
                )
            })
            .enumerate()
            .map(move |(i, (subsegment_idx, color))| {
                let end = self.fraction.start + subsegment_size * (i + 1) as f32;
                Subsegment { subsegment_idx, color, end }
            })
    }

    /// Render the segment into a list of drawable subsegments
    /// each represented as a list of points and a color,
    /// `snake_len` is used to calculate how many subsegments
    /// there should be (longer snakes have lower subsegment
    /// resolution)
    pub fn render(
        &self,
        color_resolution: usize,
        turn_fraction: f32,
    ) -> Box<dyn Iterator<Item = (Color, Vec<Point>)> + '_> {
        match self.draw_style {
            rendering::Style::Hexagon => Box::new(iter::once((
                self.segment_style.first_color(),
                HexagonSegments::render_segment(
                    self,
                    0,
                    0.0,
                    SegmentFraction::solid(),
                    RoundHeadDescription::Gone,
                ),
            ))),

            rendering::Style::Smooth => {
                let round_head = self
                    .fraction
                    .round_head_description(self.prev_fraction, self.cell_dim);

                let mut end = self.fraction.start;
                // TODO: remove collect, why do we even need the SubSegment type?
                let info: Vec<_> = self
                    .get_subsegments(color_resolution)
                    .map(move |subsegment| {
                        let start = end;
                        end = subsegment.end;
                        (subsegment.subsegment_idx, start, end, subsegment.color)
                    })
                    .collect();

                // TODO: in general, this isn't pretty
                Box::new(
                    info.into_iter()
                        .map(move |(subsegment_idx, start, end, color)| {
                            let points = SmoothSegments::render_segment(
                                self,
                                subsegment_idx,
                                turn_fraction,
                                SegmentFraction { start, end },
                                round_head,
                            );
                            (color, points)
                        }),
                )
            }
        }
    }

    /// Returns number of polygons built
    pub fn build(self, builder: &mut MeshBuilder, color_resolution: usize) -> Result<usize> {
        let mut polygons = 0;
        let turn_fraction = self.turn.fraction;
        self.render(color_resolution, turn_fraction)
            .try_for_each(|(color, points)| {
                if points.len() >= 3 {
                    polygons += 1;
                    builder
                        .polygon(DrawMode::fill(), &points, *color)
                        .map(|_| ())
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
        subsegment_idx: usize,
        turn_fraction: f32,
        fraction: SegmentFraction,
        round_head: RoundHeadDescription,
    ) -> Vec<Point>;
}
