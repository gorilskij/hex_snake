use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context,
};
use itertools::izip;

use crate::{
    app::{
        rendering::segments::{
            descriptions::{SegmentDescription, SegmentFraction, TurnDescription},
            render_hexagon,
        },
        snake::{SegmentType, Snake},
        stats::Stats,
    },
    basic::{transformations::translate, CellDim, Point},
    partial_min_max::partial_min,
};
use crate::app::game_context::GameContext;
use crate::app::app_error::{AppResult, GameResultExtension};
use crate::app::snake::{Segment, State};
use crate::app::snake::palette::SegmentStyle;

const DRAW_WHITE_AURA: bool = false;

fn build_hexagon_at(
    location: Point,
    cell_dim: CellDim,
    color: Color,
    builder: &mut MeshBuilder,
) -> AppResult {
    let mut hexagon_points = render_hexagon(cell_dim);
    translate(&mut hexagon_points, location);
    builder.polygon(DrawMode::fill(), &hexagon_points, color).into_with_trace("build_hexagon_at")?;
    Ok(())
}

fn segment_description(segment: &Segment, segment_idx: usize, snake: &Snake, frame_fraction: f32, segment_styles: &[SegmentStyle], gtx: &GameContext) -> SegmentDescription {
    let coming_from = segment.coming_from;
    let going_to = segment_idx
        .checked_sub(1)
        .map(|prev_idx| -snake.body.cells[prev_idx].coming_from)
        .unwrap_or(snake.dir());

    let location = segment.pos.to_cartesian(gtx.cell_dim);

    let fraction = match segment_idx {
        // head
        0 => {
            if let SegmentType::BlackHole { just_created: _ } = segment.segment_type {
                // never exceed 0.5 into a black hole, stay there once you get there
                if snake.visible_len() == 1 {
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
        i if i == snake.visible_len() - 1 && snake.body.grow == 0 => {
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

    SegmentDescription {
        destination: location,
        turn: TurnDescription { coming_from, going_to },
        fraction,
        draw_style: gtx.prefs.draw_style,
        segment_style: segment_styles[segment_idx],
        cell_dim: gtx.cell_dim,
    }
}

fn build_snake_body(
    snake: &mut Snake,
    frame_fraction: f32,
    subsegments_per_segment: usize,
    segment_styles: &[SegmentStyle],
    gtx: &GameContext,
    stats: &mut Stats,
    builder: &mut MeshBuilder,
) -> AppResult {
    // If the snake is guided by a search algorithm, draw the cells
    // that were searched and the path that is being followed
    if let Some(search_trace) = &snake.body.search_trace {
        let searched_cell_color = Color::from_rgb(130, 47, 5);
        let current_path_color = Color::from_rgb(97, 128, 11);
        for &point in &search_trace.cells_searched {
            build_hexagon_at(
                point.to_cartesian(gtx.cell_dim),
                gtx.cell_dim,
                searched_cell_color,
                builder,
            )?;
        }
        for &point in &search_trace.current_path {
            build_hexagon_at(
                point.to_cartesian(gtx.cell_dim),
                gtx.cell_dim,
                current_path_color,
                builder,
            )?;
        }
        stats.polygons += search_trace.cells_searched.len() + search_trace.current_path.len();
    }

    // Draw white aura around snake heads (debug)
    if DRAW_WHITE_AURA {
        for point in snake.reachable(7, gtx.board_dim) {
            build_hexagon_at(
                point.to_cartesian(gtx.cell_dim),
                gtx.cell_dim,
                Color::WHITE,
                builder,
            )?;
            stats.polygons += 1;
        }
    }

    for (segment_idx, segment) in snake.body.cells.iter().enumerate() {
        let segment_description = segment_description(segment, segment_idx, snake, frame_fraction, &segment_styles, gtx);

        // Draw body segments, turn transition for all non-head segments is 1
        stats.polygons +=
            segment_description.build(builder, subsegments_per_segment, 1.)?;
    }

    Ok(())
}

// TODO: the draw order is actually more complicated
//  heads of non-dying snakes that are going towards
//  the black hole need to be drawn on top of it but
//  those that are going away from the black hole need
//  to be drawn below it (see debug scenario 3)
// TODO: refactor this whole mess
pub fn snake_mesh(
    snakes: &mut [Snake],
    gtx: &GameContext,
    ctx: &mut Context,
    stats: &mut Stats,
) -> AppResult<Mesh> {
    stats.redrawing_snakes = true;

    let frame_fraction = gtx.frame_stamp.1;

    let mut builder = MeshBuilder::new();

    // Black holes and other heads on top of body segments,
    // crashed heads are included in other heads and are
    // drawn on top of black holes
    let mut black_holes = vec![];
    let mut other_heads = vec![];

    let mut subsegments_per_segment_all = vec![];
    let mut segment_styles_all = vec![];

    // TODO: figure out a z-index with ggez
    // find all the heads, defer drawing them
    for snake in snakes.iter_mut() {
        // Desired total number of subsegments for the whole snake
        // smaller snakes have higher resolution to show more detail
        // (this is intended to work with rainbows)
        const TOTAL_SUBSEGMENTS: usize = 250;

        // Bounds on the number of subsegments per segment to avoid
        // very high numbers of polygons or empty segments
        const MIN_SUBSEGMENTS: usize = 1;
        const MAX_SUBSEGMENTS: usize = 20;

        let subsegments_per_segment = match TOTAL_SUBSEGMENTS / snake.visible_len() {
            x if x < MIN_SUBSEGMENTS => MIN_SUBSEGMENTS,
            x if x > MAX_SUBSEGMENTS => MAX_SUBSEGMENTS,
            x => x,
        };

        if subsegments_per_segment > stats.max_subsegments_per_segment {
            stats.max_subsegments_per_segment = subsegments_per_segment;
        }

        let segment_styles = snake.palette.segment_styles(&snake.body, frame_fraction);


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

        let segment = &snake.body.cells[0];
        let segment_description = segment_description(segment, 0, snake, frame_fraction, &segment_styles, gtx);

        match segment.segment_type {
            SegmentType::BlackHole { .. } => black_holes.push((
                segment_description,
                subsegments_per_segment,
                turn,
                frame_fraction,
            )),
            _ => other_heads.push((
                segment_description,
                subsegments_per_segment,
                turn,
            )),
        }

        subsegments_per_segment_all.push(subsegments_per_segment);
        segment_styles_all.push(segment_styles);
    }

    // draw bodies of non-dying snakes first, so that black holes appear
    // on top of them
    for (snake, subsegments_per_segment, segment_styles) in
        izip!(
            snakes.iter_mut(),
            subsegments_per_segment_all.iter().copied(),
            segment_styles_all.iter(),
        ).filter(|(s, _, _)| s.state != State::Dying)
    {
        build_snake_body(snake, frame_fraction, subsegments_per_segment, segment_styles, gtx, stats,  &mut builder)?;
    }

    // draw black holes and crashed heads
    // TODO: factor out into palette
    let black_hole_color = Color::from_rgb(1, 36, 92);

    // draw the actual black holes first in case there are multiple
    // snakes falling into the same black hole (black holes in the same
    // place are still drawn multiple times)
    for (segment_description, _, _, frame_fraction) in &black_holes {
        let destination = segment_description.destination + gtx.cell_dim.center();
        let real_cell_dim;
        let SegmentFraction { start, end } = segment_description.fraction;
        if (start - end).abs() < f32::EPSILON {
            // snake has died, animate black hole out
            assert!(*frame_fraction >= 0.5, "{} < 0.5", frame_fraction);
            let animation_fraction = frame_fraction - 0.5;
            real_cell_dim = gtx.cell_dim * (1. - animation_fraction);
        } else {
            real_cell_dim = gtx.cell_dim;
        }
        stats.polygons += 1;
        builder.circle(DrawMode::fill(), destination, real_cell_dim.side, 0.1, black_hole_color)?;
    }

    // draw the heads of snakes falling into black holes
    for (segment_description, subsegments_per_segment, turn, _) in black_holes {
        stats.polygons += segment_description.build(&mut builder, subsegments_per_segment, turn)?;
    }

    // draw other bodies
    for (snake, subsegments_per_segment, segment_styles) in
        izip!(
            snakes.iter_mut(),
            subsegments_per_segment_all,
            &segment_styles_all,
        ).filter(|(s, _, _)| s.state == State::Dying)
    {
        build_snake_body(snake, frame_fraction, subsegments_per_segment, segment_styles, gtx, stats,  &mut builder)?;
    }

    // draw other heads
    for (segment_description, subsegments_per_segment, turn) in other_heads {
        stats.polygons += segment_description.build(&mut builder, subsegments_per_segment, turn)?;
    }

    builder.build(ctx).into_with_trace("snake_mesh")
}
