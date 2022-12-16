use ggez::graphics::{DrawMode, MeshBuilder};
use hsl::HSL;
use itertools::Itertools;

use crate::basic::transformations::{flip_horizontally, rotate_clockwise, translate};
use crate::basic::{Dir, Point};
use crate::color::oklab::OkLab;
use crate::color::to_color::ToColor;
use crate::color::Color;
use crate::error::{Error, ErrorConversion, Result};
use crate::rendering;
use crate::rendering::segments::descriptions::{
    SegmentDescription, SegmentFraction, TurnDirection, TurnType,
};
use crate::rendering::segments::hexagon_segments::HexagonSegments;
use crate::rendering::segments::smooth_segments::SmoothSegments;
use crate::snake::palette::SegmentStyle;

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
    fn get_subsegments(&self, num_subsegments: usize) -> Vec<Subsegment> {
        if self.draw_style == rendering::Style::Hexagon {
            unreachable!("hexagon segments don't support gradients")
        }

        if num_subsegments == 1 {
            return vec![Subsegment {
                color: self.segment_style.first_color(),
                end: self.fraction.end,
            }];
        }

        let SegmentFraction { start, end } = self.fraction;
        let segment_size = self.fraction.end - self.fraction.start;

        // TODO: less collecting, keep things as iterators
        // gradients exclude the end color because this is the same as the start color of the next segment
        let colors = match self.segment_style {
            SegmentStyle::Solid(color) => vec![color],
            SegmentStyle::RGBGradient { start_color, end_color } => {
                // TODO: factor out this code
                let start_subsegment = (num_subsegments as f32 * start) as usize;
                let end_subsegment = (num_subsegments as f32 * end).ceil() as usize;

                (start_subsegment..end_subsegment)
                    .map(|f| {
                        let f = f as f64 / num_subsegments as f64;
                        f * start_color + (1. - f) * end_color
                    })
                    .collect()
            }
            SegmentStyle::HSLGradient { start_hue, end_hue, lightness } => {
                let start_subsegment = (num_subsegments as f32 * start) as usize;
                let end_subsegment = (num_subsegments as f32 * end).ceil() as usize;
                (start_subsegment..end_subsegment)
                    .map(|f| {
                        let f = f as f64 / num_subsegments as f64;
                        HSL {
                            h: f * start_hue + (1. - f) * end_hue,
                            s: 1.,
                            l: lightness,
                        }
                        .to_color()
                    })
                    .collect()
            }
            SegmentStyle::OkLabGradient { start_hue, end_hue, lightness } => {
                let start_subsegment = (num_subsegments as f32 * start) as usize;
                let end_subsegment = (num_subsegments as f32 * end).ceil() as usize;
                (start_subsegment..end_subsegment)
                    .map(|f| {
                        let f = f as f64 / num_subsegments as f64;
                        OkLab::from_lch(lightness, 0.5, f * start_hue + (1. - f) * end_hue)
                            .to_color()
                    })
                    .collect()
            }
        };

        // Can't tell if it's more inefficient to run dedup each time or
        // occasionally generate some extra segments
        // colors.dedup();

        // the actual number of subsegments (partial segments will
        //  have fewer than expected)
        let real_num_subsegments = colors.len();
        let subsegment_size = segment_size / real_num_subsegments as f32;

        colors
            .into_iter()
            .enumerate()
            .map(|(i, color)| {
                let end = self.fraction.start + subsegment_size * (i + 1) as f32;
                Subsegment { color, end }
            })
            .collect_vec()
    }

    /// Render the segment into a list of drawable subsegments
    /// each represented as a list of points and a color,
    /// `snake_len` is used to calculate how many subsegments
    /// there should be (longer snakes have lower subsegment
    /// resolution)
    pub fn render(
        mut self,
        color_resolution: usize,
        turn_fraction: f32,
    ) -> Vec<(Color, Vec<Point>)> {
        match self.draw_style {
            rendering::Style::Hexagon => vec![(
                self.segment_style.first_color(),
                HexagonSegments::render_segment(&self, turn_fraction, SegmentFraction::solid()),
            )],
            rendering::Style::Smooth => {
                let mut end = self.fraction.start;
                self.get_subsegments(color_resolution)
                    .into_iter()
                    .map(|subsegment| {
                        let start = end;
                        end = subsegment.end;
                        let points = SmoothSegments::render_segment(
                            &self,
                            turn_fraction,
                            SegmentFraction { start, end },
                        );
                        (subsegment.color, points)
                    })
                    .collect()
            }
        }
    }

    /// Returns number of polygons built
    pub fn build(self, builder: &mut MeshBuilder, color_resolution: usize) -> Result<usize> {
        let mut polygons = 0;
        let turn_fraction = self.turn.fraction;
        for (color, points) in self.render(color_resolution, turn_fraction) {
            builder
                .polygon(DrawMode::fill(), &points, *color)
                .map_err(Error::from)
                .with_trace_step("SegmentDescription::build")?;
            polygons += 1;
        }
        Ok(polygons)
    }
}

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
    ) -> Vec<Point> {
        use TurnDirection::*;
        use TurnType::*;

        let mut segment;
        match description.turn.turn_type() {
            Straight => segment = Self::render_default_straight_segment(&description, fraction),
            Blunt(turn_direction) | Sharp(turn_direction) => {
                segment =
                    Self::render_default_curved_segment(&description, turn_fraction, fraction);
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
