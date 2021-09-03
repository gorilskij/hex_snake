use crate::{
    app::{
        game::{Game, Stats},
        snake::{
            rendering::{
                descriptions::{SegmentDescription, SegmentFraction, TurnDescription},
                render_hexagon,
            }, SegmentType,
        },
    },
    basic::transformations::translate,
};
use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};

use crate::basic::{CellDim, Point};



const DRAW_WHITE_AURA: bool = false;

fn build_hexagon_at(location: Point, cell_dim: CellDim, color: Color, builder: &mut MeshBuilder) -> GameResult {
    let mut hexagon_points = render_hexagon(cell_dim);
    translate(&mut hexagon_points, location);
    builder.polygon(
        DrawMode::fill(),
        &hexagon_points,
        color,
    )?;
    Ok(())
}

impl Game {
    pub(in crate::app::game) fn snake_mesh(
        &mut self,
        ctx: &mut Context,
        stats: &mut Stats,
    ) -> GameResult<Mesh> {
        stats.redrawing_snakes = true;

        let mut builder = MeshBuilder::new();

        // Black holes and other heads on top of body segments,
        // crashed heads are included in other heads and are
        // drawn on top of black holes
        let mut black_holes = vec![];
        let mut other_heads = vec![];

        let frame_stamp = self.control.frame_stamp();
        let frame_frac = frame_stamp.1;

        // Draw bodies
        for snake_idx in 0..self.snakes.len() {
            let (snake, other_snakes) = Self::split_snakes_mut(&mut self.snakes, snake_idx);

            // Desired total number of subsegments for the whole snake
            // smaller snakes have higher resolution to show more detail
            // (this is intended to work with rainbows)
            const TOTAL_SUBSEGMENTS: usize = 250;

            // Bounds on the number of subsegments per segment to avoid
            // very high numbers of polygons or empty segments
            const MIN_SUBSEGMENTS: usize = 1;
            const MAX_SUBSEGMENTS: usize = 20;

            let subsegments_per_segment = match TOTAL_SUBSEGMENTS / snake.len() {
                x if x < MIN_SUBSEGMENTS => MIN_SUBSEGMENTS,
                x if x > MAX_SUBSEGMENTS => MAX_SUBSEGMENTS,
                x => x,
            };

            if subsegments_per_segment > stats.max_subsegments_per_segment {
                stats.max_subsegments_per_segment = subsegments_per_segment;
            }

            // update the direction of the snake early
            // to see it turning as soon as possible,
            // this could happen in the middle of a
            // game frame
            snake.update_dir(other_snakes, &self.apples, self.dim, frame_stamp);

            // If the snake is guided by a search algorithm, draw the cells
            // that were searched and the path that is being followed
            if let Some(search_trace) = &snake.body.search_trace {
                let searched_cell_color = Color::from_rgb(130, 47, 5);
                let current_path_color = Color::from_rgb(97, 128, 11);
                for &point in &search_trace.cells_searched {
                    build_hexagon_at(point.to_point(self.cell_dim), self.cell_dim, searched_cell_color, &mut builder)?;
                }
                for &point in &search_trace.current_path {
                    build_hexagon_at(point.to_point(self.cell_dim), self.cell_dim, current_path_color, &mut builder)?;
                }
                stats.polygons += search_trace.cells_searched.len() + search_trace.current_path.len();
            }


            // Draw white aura around snake heads (debug)
            if DRAW_WHITE_AURA {
                for point in snake.reachable(7, self.dim) {
                    build_hexagon_at(point.to_point(self.cell_dim), self.cell_dim, Color::WHITE, &mut builder)?;
                    stats.polygons += 1;
                }
            }


            let segment_styles = snake.palette.segment_styles(&snake.body, frame_frac);
            for (segment_idx, segment) in snake.body.cells.iter().enumerate() {
                let coming_from = segment.coming_from;
                let going_to = segment_idx
                    .checked_sub(1)
                    .map(|prev_idx| -snake.body.cells[prev_idx].coming_from)
                    .unwrap_or(snake.dir());

                if coming_from == going_to {
                    // TODO: diagnose this bug
                    panic!("180° turn ({:?} -> {:?}) at idx {} of snake at idx {}, segment_type: {:?}", coming_from, going_to, segment_idx, snake_idx, segment.typ);
                }

                let location = segment.pos.to_point(self.cell_dim);

                let fraction = match segment_idx {
                    // head
                    0 => SegmentFraction::appearing(frame_frac),
                    // tail
                    i if i == snake.len() - 1 && snake.body.grow == 0 => {
                        if let SegmentType::Eaten { original_food, food_left } = segment.typ {
                            let frac = ((original_food - food_left) as f32 + frame_frac)
                                / (original_food + 1) as f32;
                            SegmentFraction::disappearing(frac)
                        } else {
                            SegmentFraction::disappearing(frame_frac)
                        }
                    }
                    // body
                    _ => SegmentFraction::solid(),
                };

                let segment_description = SegmentDescription {
                    destination: location,
                    turn: TurnDescription { coming_from, going_to },
                    fraction,
                    draw_style: self.prefs.draw_style,
                    segment_style: segment_styles[segment_idx],
                    cell_dim: self.cell_dim,
                };

                if segment_idx == 0 {
                    // Defer drawing heads, calculate smooth turn
                    let turn = snake.body.turn_start.map(|(_, start_frame_frac)| {
                        let max = 1. - start_frame_frac;
                        let covered = frame_frac - start_frame_frac;
                        covered / max
                    }).unwrap_or(1.);

                    match segment.typ {
                        SegmentType::BlackHole => black_holes.push((segment_description, subsegments_per_segment, turn)),
                        _ => other_heads.push((segment_description, subsegments_per_segment, turn)),
                    }
                } else {
                    // Draw body segments, turn transition for all non-head segments is 1
                    stats.polygons += segment_description.build(&mut builder, subsegments_per_segment, 1.)?;
                }
            }
        }


        // Draw black holes and crashed heads
        let black_hole_color = Color::from_rgb(1, 36, 92);
        for (segment_description, subsegments_per_segment, turn) in black_holes {
            build_hexagon_at(
                segment_description.destination,
                self.cell_dim,
                black_hole_color,
                &mut builder,
            )?;
            stats.polygons += 1;
            stats.polygons += segment_description.build(&mut builder, subsegments_per_segment, turn)?;
        }
        for (segment_description, subsegments_per_segment, turn) in other_heads {
            stats.polygons += segment_description.build(&mut builder, subsegments_per_segment, turn)?;
        }

        builder.build(ctx)
    }
}
