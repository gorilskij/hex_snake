use ggez::{
    graphics::{Color, DrawMode, MeshBuilder},
    GameResult,
};
use hsl::HSL;
use itertools::Itertools;

use crate::{
    app::snake::{
        palette::SegmentStyle,
        rendering::{
            descriptions::{SegmentDescription, SegmentFraction, TurnDirection, TurnType},
            hexagon_segments::HexagonSegments,
            rough_segments::RoughSegments,
            smooth_segments::SmoothSegments,
        },
    },
    basic::{
        transformations::{flip_horizontally, rotate_clockwise, translate},
        CellDim, Dir, DrawStyle, Point,
    },
};

impl SegmentDescription {
    /// Split a single segment description into `n` subsegments,
    /// this is used to assign a solid color to each subsegment and thus
    /// simulate a smooth gradient
    fn split_into_subsegments(self, num_subsegments: usize) -> Vec<Self> {
        if num_subsegments == 1 {
            return vec![self]
        }

        let SegmentFraction { start, end } = self.fraction;
        let segment_size = self.fraction.end - self.fraction.start;

        // gradients exclude the end color because this is the same as the start color of the next segment
        let colors = match self.segment_style {
            SegmentStyle::Solid(color) => vec![color],
            SegmentStyle::RGBGradient {
                start_rgb: (r1, g1, b1),
                end_rgb: (r2, g2, b2),
            } => {
                let r1 = r1 as f64;
                let g1 = g1 as f64;
                let b1 = b1 as f64;
                let r2 = r2 as f64;
                let g2 = g2 as f64;
                let b2 = b2 as f64;

                let start_subsegment = (num_subsegments as f32 * start) as usize;
                let end_subsegment = (num_subsegments as f32 * end).ceil() as usize;

                (start_subsegment..end_subsegment)
                    .map(|f| {
                        let f = f as f64 / num_subsegments as f64;
                        Color::from_rgb(
                            (f * r1 + (1. - f) * r2) as u8,
                            (f * g1 + (1. - f) * g2) as u8,
                            (f * b1 + (1. - f) * b2) as u8,
                        )
                    })
                    .collect()
            }
            SegmentStyle::HSLGradient {
                start_hue,
                end_hue,
                lightness,
            } => {
                let start_subsegment = (num_subsegments as f32 * start) as usize;
                let end_subsegment = (num_subsegments as f32 * end).ceil() as usize;
                (start_subsegment..end_subsegment)
                    .map(|f| {
                        let f = f as f64 / num_subsegments as f64;
                        Color::from(
                            HSL {
                                h: f * start_hue + (1. - f) * end_hue,
                                s: 1.,
                                l: lightness,
                            }
                            .to_rgb(),
                        )
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
                let start = self.fraction.start + subsegment_size * i as f32;
                let end = start + subsegment_size;
                Self {
                    destination: self.destination,
                    turn: self.turn,
                    fraction: SegmentFraction { start, end },
                    draw_style: self.draw_style,
                    segment_style: SegmentStyle::Solid(color),
                    cell_dim: self.cell_dim,
                }
            })
            .collect_vec()
    }

    // subsegments in particular are expected to be of the Solid variant
    fn as_solid_color(&self) -> Color {
        match &self.segment_style {
            SegmentStyle::Solid(color) => *color,
            SegmentStyle::RGBGradient { start_rgb, .. } => Color::from(*start_rgb),
            SegmentStyle::HSLGradient { start_hue, lightness, .. } => {
                let hsl = HSL {
                    h: *start_hue,
                    s: 1.,
                    l: *lightness,
                };
                Color::from(hsl.to_rgb())
            }
        }
    }

    /// Render the segment into a list of drawable subsegments
    /// each represented as a list of points and a color,
    /// `snake_len` is used to calculate how many subsegments
    /// there should be (longer snakes have lower subsegment
    /// resolution)
    pub fn render(mut self, subsegments_per_segment: usize) -> Vec<(Color, Vec<Point>)> {
        let subsegments = if self.draw_style == DrawStyle::Hexagon {
            // hexagon segments don't support gradients
            self.fraction = SegmentFraction::solid();
            self.segment_style = self.segment_style.into_solid();
            vec![self]
        } else {
            self.split_into_subsegments(subsegments_per_segment)
        };

        // self.fraction = SegmentFraction::solid();
        // self.segment_style = self.segment_style.into_solid();
        // let subsegments = vec![self];

        subsegments
            .into_iter()
            .map(|subsegment| {
                let color = subsegment.as_solid_color();
                let points = match subsegment.draw_style {
                    DrawStyle::Hexagon => HexagonSegments::render_segment(subsegment),
                    DrawStyle::Rough => RoughSegments::render_segment(subsegment),
                    DrawStyle::Smooth => SmoothSegments::render_segment(subsegment),
                };
                (color, points)
            })
            .collect()
    }

    pub fn build(self, builder: &mut MeshBuilder, subsegments_per_segment: usize) -> GameResult {
        for (color, points) in self.render(subsegments_per_segment) {
            builder.polygon(DrawMode::fill(), &points, color)?;
        }
        Ok(())
    }
}

/// The `render_default_*` functions are without position or rotation, they simply generate the points that correspond to a type of turn (straight, blunt, or sharp)
pub trait SegmentRenderer {
    /// Default straight segment coming from above (U) and going down (D)
    fn render_default_straight(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point>;

    /// Default blunt segment coming from above (U) and going down-right (DR)
    fn render_default_blunt(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point>;

    /// Default sharp segment coming from above (U) and going up-right (UR)
    fn render_default_sharp(cell_dim: CellDim, fraction: SegmentFraction) -> Vec<Point>;

    /// Turns a default segment into one that is ready to be printed
    /// adding position and rotating and reflecting to fit the desired
    /// from and to directions
    fn render_segment(description: SegmentDescription) -> Vec<Point> {
        use TurnDirection::*;
        use TurnType::*;

        let mut segment = match description.turn.turn_type() {
            Straight => Self::render_default_straight(description.cell_dim, description.fraction),
            Blunt(turn_direction) => {
                let mut default_segment =
                    Self::render_default_blunt(description.cell_dim, description.fraction);
                if turn_direction == Clockwise {
                    flip_horizontally(&mut default_segment, description.cell_dim.center().x);
                }
                default_segment
            }
            Sharp(turn_direction) => {
                let mut default_segment =
                    Self::render_default_sharp(description.cell_dim, description.fraction);
                if turn_direction == Clockwise {
                    flip_horizontally(&mut default_segment, description.cell_dim.center().x);
                }
                default_segment
            }
        };

        let rotation_angle = Dir::U.clockwise_angle_to(description.turn.coming_from);
        if rotation_angle != 0. {
            rotate_clockwise(&mut segment, description.cell_dim.center(), rotation_angle);
        }

        translate(&mut segment, description.destination);

        segment
    }
}
