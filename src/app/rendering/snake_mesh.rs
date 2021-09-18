use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context, GameResult,
};

use crate::{
    app::{
        rendering,
        snake::{
            render::{
                descriptions::{SegmentDescription, SegmentFraction, TurnDescription},
                render_hexagon,
            },
            SegmentType, Snake,
        },
        stats::Stats,
    },
    basic::{transformations::translate, CellDim, FrameStamp, HexDim, Point},
};

use crate::partial_min_max::partial_min;

const DRAW_WHITE_AURA: bool = false;

fn build_hexagon_at(
    location: Point,
    cell_dim: CellDim,
    color: Color,
    builder: &mut MeshBuilder,
) -> GameResult {
    let mut hexagon_points = render_hexagon(cell_dim);
    translate(&mut hexagon_points, location);
    builder.polygon(DrawMode::fill(), &hexagon_points, color)?;
    Ok(())
}

pub fn snake_mesh(
    snakes: &mut [Snake],
    frame_stamp: FrameStamp,
    board_dim: HexDim,
    cell_dim: CellDim,
    draw_style: rendering::Style,
    ctx: &mut Context,
    stats: &mut Stats,
) -> GameResult<Mesh> {
    stats.redrawing_snakes = true;

    let frame_fraction = frame_stamp.1;

    let mut builder = MeshBuilder::new();

    // Black holes and other heads on top of body segments,
    // crashed heads are included in other heads and are
    // drawn on top of black holes
    let mut black_holes = vec![];
    let mut other_heads = vec![];

    // Draw bodies
    for snake in snakes {
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

        // If the snake is guided by a search algorithm, draw the cells
        // that were searched and the path that is being followed
        if let Some(search_trace) = &snake.body.search_trace {
            let searched_cell_color = Color::from_rgb(130, 47, 5);
            let current_path_color = Color::from_rgb(97, 128, 11);
            for &point in &search_trace.cells_searched {
                build_hexagon_at(
                    point.to_cartesian(cell_dim),
                    cell_dim,
                    searched_cell_color,
                    &mut builder,
                )?;
            }
            for &point in &search_trace.current_path {
                build_hexagon_at(
                    point.to_cartesian(cell_dim),
                    cell_dim,
                    current_path_color,
                    &mut builder,
                )?;
            }
            stats.polygons += search_trace.cells_searched.len() + search_trace.current_path.len();
        }

        // Draw white aura around snake heads (debug)
        if DRAW_WHITE_AURA {
            for point in snake.reachable(7, board_dim) {
                build_hexagon_at(
                    point.to_cartesian(cell_dim),
                    cell_dim,
                    Color::WHITE,
                    &mut builder,
                )?;
                stats.polygons += 1;
            }
        }

        let segment_styles = snake.palette.segment_styles(&snake.body, frame_fraction);
        for (segment_idx, segment) in snake.body.cells.iter().enumerate() {
            let coming_from = segment.coming_from;
            let going_to = segment_idx
                .checked_sub(1)
                .map(|prev_idx| -snake.body.cells[prev_idx].coming_from)
                .unwrap_or(snake.dir());

            if coming_from == going_to {
                // TODO: diagnose this bug
                panic!(
                    "180° turn ({:?} -> {:?}) at idx {}, segment_type: {:?}",
                    coming_from, going_to, segment_idx, segment.segment_type
                );
            }

            let location = segment.pos.to_cartesian(cell_dim);

            let fraction = match segment_idx {
                // head
                0 => {
                    if let SegmentType::BlackHole { just_created: _ } = segment.segment_type {
                        // never exceed 0.5 into a black hole, stay there once you get there
                        if snake.len() == 1 {
                            // also tail
                            SegmentFraction {
                                start: partial_min(frame_fraction, 0.5).unwrap(),
                                end: 0.5,
                            }
                        } else if snake.body.missing_front > 0 {
                            SegmentFraction::appearing(0.5)
                        } else {
                            SegmentFraction::appearing(partial_min(frame_fraction, 0.5).unwrap())
                        }
                    } else {
                        SegmentFraction::appearing(frame_fraction)
                    }
                }
                // tail
                i if i == snake.len() - 1 && snake.body.grow == 0 => {
                    if let SegmentType::Eaten { original_food, food_left } = segment.segment_type {
                        let frac = ((original_food - food_left) as f32 + frame_fraction)
                            / (original_food + 1) as f32;
                        SegmentFraction::disappearing(frac)
                    } else {
                        SegmentFraction::disappearing(frame_fraction)
                    }
                }
                // body
                _ => SegmentFraction::solid(),
            };

            let segment_description = SegmentDescription {
                destination: location,
                turn: TurnDescription { coming_from, going_to },
                fraction,
                draw_style,
                segment_style: segment_styles[segment_idx],
                cell_dim,
            };

            if segment_idx == 0 {
                // Defer drawing heads, calculate smooth turn
                let turn = snake
                    .body
                    .turn_start
                    .map(|(_, start_frame_fraction)| {
                        let max = 1. - start_frame_fraction;
                        let covered = frame_fraction - start_frame_fraction;
                        let linear = covered / max;
                        // apply easing
                        ezing::sine_inout(linear)
                    })
                    .unwrap_or(1.);

                match segment.segment_type {
                    SegmentType::BlackHole { .. } => black_holes.push((
                        segment_description,
                        subsegments_per_segment,
                        turn,
                        frame_fraction,
                    )),
                    _ => other_heads.push((segment_description, subsegments_per_segment, turn)),
                }
            } else {
                // Draw body segments, turn transition for all non-head segments is 1
                stats.polygons +=
                    segment_description.build(&mut builder, subsegments_per_segment, 1.)?;
            }
        }
    }

    // Draw black holes and crashed heads
    let black_hole_color = Color::from_rgb(1, 36, 92);

    // draw the actual black holes first in case there are multiple
    // snakes falling into the same black hole (black holes in the same
    // place are still drawn multiple times)
    for (segment_description, _, _, frame_fraction) in &black_holes {
        let destination;
        let real_cell_dim;
        if segment_description.fraction.start == segment_description.fraction.end {
            // snake has died, animate black hole out
            assert!(*frame_fraction >= 0.5, "{} < 0.5", frame_fraction);
            let animation_fraction = frame_fraction - 0.5;
            destination = segment_description.destination + cell_dim.center() * animation_fraction;
            real_cell_dim = cell_dim * (1. - animation_fraction);
        } else {
            destination = segment_description.destination;
            real_cell_dim = cell_dim;
        }
        build_hexagon_at(
            destination,
            real_cell_dim,
            black_hole_color,
            &mut builder,
        )?;
    }

    for (segment_description, subsegments_per_segment, turn, _) in black_holes {
        stats.polygons += 1;
        stats.polygons += segment_description.build(&mut builder, subsegments_per_segment, turn)?;
    }
    for (segment_description, subsegments_per_segment, turn) in other_heads {
        stats.polygons += segment_description.build(&mut builder, subsegments_per_segment, turn)?;
    }

    builder.build(ctx)
}
