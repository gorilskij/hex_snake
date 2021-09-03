use crate::{
    app::{
        game::{Game, Stats},
        snake::{
            palette::SegmentStyle,
            rendering::{
                descriptions::{SegmentDescription, SegmentFraction, TurnDescription},
                render_hexagon,
            },
            Segment, SegmentType, SnakeState,
        },
    },
    basic::transformations::translate,
};
use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};

impl Game {
    pub(in crate::app::game) fn snake_mesh(
        &mut self,
        frame_frac: f32,
        ctx: &mut Context,
        stats: &mut Stats,
    ) -> GameResult<Mesh> {
        stats.redrawing_snakes = true;

        let mut builder = MeshBuilder::new();

        // to be drawn later (potentially on top of body segments)
        let mut heads = vec![];

        let frame_frac = self.control.frame_fraction();

        // draw bodies
        for snake_idx in 0..self.snakes.len() {
            let (snake, other_snakes) = Self::split_snakes_mut(&mut self.snakes, snake_idx);

            // desired total number of subsegments for the whole snake
            //  smaller snakes have higher resolution to show more detail
            //  (this is intended to work with rainbows)
            const TOTAL_SUBSEGMENTS: usize = 250;

            // some bounds on the number of subsegments per segment
            //  to avoid very high numbers of polygons or empty segments
            const MIN_SUBSEGMENTS: usize = 1;
            const MAX_SUBSEGMENTS: usize = 20;

            let subsegments_per_segment = match TOTAL_SUBSEGMENTS / snake.len() {
                x if x < MIN_SUBSEGMENTS => MIN_SUBSEGMENTS,
                x if x > MAX_SUBSEGMENTS => MAX_SUBSEGMENTS,
                x => x,
            };

            if subsegments_per_segment > stats.subsegments_per_segment {
                stats.subsegments_per_segment = subsegments_per_segment;
            }

            // update the direction of the snake early
            // to see it turning as soon as possible,
            // this could happen in the middle of a
            // game frame
            snake.update_dir(other_snakes, &self.apples, self.dim);

            let len = snake.len();

            // draw white aura around snake heads (debug)
            // for pos in snake.reachable(7, self.dim) {
            //     let dest = pos.to_point(self.cell_dim);
            //     let mut points = render_hexagon(self.cell_dim);
            //     translate(&mut points, dest);
            //     builder.polygon(DrawMode::fill(), &points, Color::WHITE)?;
            // }

            let segment_styles = snake.palette.segment_styles(&snake.body, frame_frac);
            for (seg_idx, segment) in snake.body.cells.iter().enumerate() {
                // previous = towards head
                // next = towards tail

                let coming_from = segment.coming_from;
                let going_to = seg_idx
                    .checked_sub(1)
                    .map(|prev_idx| -snake.body.cells[prev_idx].coming_from)
                    .unwrap_or_else(|| snake.dir());

                if seg_idx == 0 && matches!(snake.state, SnakeState::Crashed | SnakeState::Dying) {
                    assert!(
                        matches!(segment.typ, SegmentType::Crashed | SegmentType::BlackHole),
                        "head of type {:?} in snake in state {:?}",
                        segment.typ,
                        snake.state
                    );

                    let turn_transition = snake.body.turn_start.map(|(_, start_frame_frac)| {
                        let max = 1. - start_frame_frac;
                        let done = frame_frac - start_frame_frac;
                        done / max
                    }).unwrap_or(1.);

                    // draw head separately
                    heads.push((
                        *segment,
                        coming_from,
                        going_to,
                        segment_styles[seg_idx],
                        subsegments_per_segment,
                        turn_transition,
                    ));
                    continue;
                }

                let location = segment.pos.to_point(self.cell_dim);

                let fraction = match seg_idx {
                    0 => SegmentFraction::appearing(frame_frac),
                    i if i == len - 1 && snake.body.grow == 0 => {
                        if let SegmentType::Eaten { original_food, food_left } = segment.typ {
                            let frac = ((original_food - food_left) as f32 + frame_frac)
                                / (original_food + 1) as f32;
                            SegmentFraction::disappearing(frac)
                        } else {
                            SegmentFraction::disappearing(frame_frac)
                        }
                    }
                    _ => SegmentFraction::solid(),
                };

                let segment = SegmentDescription {
                    destination: location,
                    turn: TurnDescription { coming_from, going_to },
                    fraction,
                    draw_style: self.prefs.draw_style,
                    segment_style: segment_styles[seg_idx],
                    cell_dim: self.cell_dim,
                };

                // turn transition for all non-head segments is 1
                for (color, points) in segment.render(subsegments_per_segment, 1.) {
                    builder.polygon(DrawMode::fill(), &points, color)?;
                    stats.total_subsegments += 1;
                }
            }
        }

        // draw heads
        for (
            segment,
            coming_from,
            going_to,
            seg_style,
            subsegments_per_segment,
            turn_transition,
        ) in heads {
            let Segment { pos, typ, .. } = segment;
            let location = pos.to_point(self.cell_dim);

            let segment_color = match seg_style {
                SegmentStyle::Solid(color) => color,
                _ => unimplemented!(),
            };

            let head_description = SegmentDescription {
                destination: location,
                turn: TurnDescription { coming_from, going_to },
                fraction: SegmentFraction::appearing(0.5),
                draw_style: self.prefs.draw_style,
                segment_style: SegmentStyle::Solid(segment_color),
                cell_dim: self.cell_dim,
            };
            match typ {
                SegmentType::BlackHole => {
                    let hexagon_color = Color::from_rgb(1, 36, 92);
                    let mut hexagon_points = render_hexagon(self.cell_dim);
                    translate(&mut hexagon_points, location);
                    builder.polygon(DrawMode::fill(), &hexagon_points, hexagon_color)?;
                    head_description.build(&mut builder, subsegments_per_segment, turn_transition)?;
                }
                SegmentType::Crashed => {
                    head_description.build(&mut builder, subsegments_per_segment, turn_transition)?;
                }
                _ => unreachable!(
                    "head segment of type {:?} should not have been queued to be drawn separately",
                    typ
                ),
            }
        }

        // draw A* plan
        #[cfg(feature = "show_search_path")]
        unsafe {
            if let Some(seen) = &crate::app::snake::controller::ETHEREAL_SEEN {
                for point in seen {
                    let mut hexagon_points = render_hexagon(self.cell_dim);
                    let location = point.to_point(self.cell_dim);
                    translate(&mut hexagon_points, location);
                    builder.polygon(
                        DrawMode::fill(),
                        &hexagon_points,
                        Color::from_rgb(130, 47, 5),
                    )?;
                }
            }

            if let Some(path) = &crate::app::snake::controller::ETHEREAL_PATH {
                for point in path {
                    let mut hexagon_points = render_hexagon(self.cell_dim);
                    let location = point.to_point(self.cell_dim);
                    translate(&mut hexagon_points, location);
                    builder.polygon(
                        DrawMode::fill(),
                        &hexagon_points,
                        Color::from_rgb(97, 128, 11),
                    )?;
                }
            }
        }

        builder.build(ctx)
    }
}
