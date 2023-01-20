use ggez::graphics::{DrawMode, MeshBuilder};
use std::iter;

use crate::basic::transformations::{flip_horizontally, rotate_clockwise, translate};
use crate::basic::{Dir, Point};
use crate::color::Color;
use crate::error::{Error, ErrorConversion, Result};
use crate::rendering;
use crate::rendering::segments::descriptions::{
    SegmentDescription, SegmentFraction, TurnDirection, TurnType,
};
use crate::rendering::segments::hexagon_segments::HexagonSegments;
use crate::rendering::segments::smooth_segments::SmoothSegments;

struct Subsegment {
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
        (start_subsegment..end_subsegment)
            .map(move |subsegment| get_color(subsegment as f64 / num_subsegments as f64))
            .enumerate()
            .map(move |(i, color)| {
                let end = self.fraction.start + subsegment_size * (i + 1) as f32;
                Subsegment { color, end }
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
        // TODO: turn this into an iterator
        /// return (previous, current, next) at each step with previous and next if available
        /// e.g. [1,2,3] -> (None, 1, Some(2)), (Some(1), 2, Some(3)), (Some(2), 3, None)
        fn windows3<T: Copy>(vec: Vec<T>) -> impl Iterator<Item = (Option<T>, T, Option<T>)> {
            let prev = iter::once(None)
                .chain(vec.clone()
                    .into_iter()
                    .map(Some));

            let next = vec
                .clone()
                .into_iter()
                .skip(1)
                .map(Some)
                .chain(iter::once(None));

            prev.zip(vec.into_iter())
                .zip(next)
                .map(|((prev, item), next)| (prev, item, next))
        }

        match self.draw_style {
            rendering::Style::Hexagon => Box::new(iter::once((
                self.segment_style.first_color(),
                HexagonSegments::render_segment(
                    self,
                    turn_fraction,
                    SegmentFraction::solid(),
                    None,
                    None,
                ),
            ))),

            rendering::Style::Smooth => {
                let mut end = self.fraction.start;
                // TODO: remove collect and rewrite this mess
                let info: Vec<_> = self
                    .get_subsegments(color_resolution)
                    .map(move |subsegment| {
                        let start = end;
                        end = subsegment.end;
                        (start, end, subsegment.color)
                    })
                    .collect();

                dbg!(&info);

                // TODO: in general, this isn't pretty
                Box::new({
                    windows3(info).map(move |(previous, current, next)| {
                        let previous_fraction =
                            previous.map(|(start, end, _)| SegmentFraction { start, end });

                        let fraction = SegmentFraction { start: current.0, end: current.1 };

                        let next_fraction =
                            next.map(|(start, end, _)| SegmentFraction { start, end });

                        let points = SmoothSegments::render_segment(
                            self,
                            turn_fraction,
                            fraction,
                        );
                        (current.2, points)
                    })
                })
            }
        }
    }

    /// Returns number of polygons built
    pub fn build(self, builder: &mut MeshBuilder, color_resolution: usize) -> Result<usize> {
        let mut polygons = 0;
        let turn_fraction = self.turn.fraction;
        self.render(color_resolution, turn_fraction)
            .try_for_each(|(color, points)| {
                polygons += 1;
                builder
                    .polygon(DrawMode::fill(), &points, *color)
                    .map(|_| ())
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
    /// Render a straight segment in the default orientation,
    /// coming from above (U) and going down (D)
    fn render_default_straight_segment(
        description: &SegmentDescription,
        fraction: SegmentFraction,
        // fraction of the segment after this one
        next_fraction: Option<SegmentFraction>,
        // fraction of the segment before this one
        previous_fraction: Option<SegmentFraction>,
    ) -> Vec<Point>;

    /// Render a curved segment in the default orientation,
    /// a blunt segment coming from above (U) and going down-right (Dr)
    /// or a sharp segment coming from above (U) and going up-right (Ur)
    ///
    /// `turn` describes how far along the segment is on its turn,
    /// a value of 0 means the segment is straight, a value of 1 means
    /// the turn is complete
    fn render_default_curved_segment(
        description: &SegmentDescription,
        turn_fraction: f32,
        fraction: SegmentFraction,
    ) -> Vec<Point>;

    /// Render a segment, rotate it and reflect it to match the desired
    /// coming-from and going-to directions, and translate it to match
    /// the desired position
    fn render_segment(
        description: &SegmentDescription,
        turn_fraction: f32,
        fraction: SegmentFraction,
        next_fraction: Option<SegmentFraction>,
        previous_fraction: Option<SegmentFraction>,
    ) -> Vec<Point> {
        use TurnDirection::*;
        use TurnType::*;

        let mut segment;
        match description.turn.turn_type() {
            Straight => {
                segment = Self::render_default_straight_segment(
                    description,
                    fraction,
                    next_fraction,
                    previous_fraction,
                )
            }
            Blunt(turn_direction) | Sharp(turn_direction) => {
                segment = Self::render_default_curved_segment(description, turn_fraction, fraction);
                if turn_direction == Clockwise {
                    flip_horizontally(&mut segment, description.cell_dim.center().x);
                }
            }
        }

        let rotation_angle = Dir::U.clockwise_angle_to(description.turn.coming_from);
        if rotation_angle != 0. {
            rotate_clockwise(&mut segment, description.cell_dim.center(), rotation_angle);
        }

        translate(&mut segment, description.destination);

        segment
    }
}
