use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder},
    Context,
};
use rayon::prelude::*;
use static_assertions::assert_impl_all;
use std::cmp::Ordering;

use crate::{
    app::{
        app_error::{AppError, AppErrorConversion, AppResult},
        game_context::GameContext,
        rendering::segments::{
            descriptions::{SegmentDescription, SegmentFraction, TurnDescription},
            render_hexagon,
        },
        snake::{palette::SegmentStyle, Segment, SegmentType, Snake},
        stats::Stats,
    },
    basic::{transformations::translate, CellDim, Point},
    support::partial_min_max::partial_min,
};

fn build_hexagon_at(
    location: Point,
    cell_dim: CellDim,
    color: Color,
    builder: &mut MeshBuilder,
) -> AppResult {
    let mut hexagon_points = render_hexagon(cell_dim);
    translate(&mut hexagon_points, location);
    builder
        .polygon(DrawMode::fill(), &hexagon_points, color)
        .map_err(AppError::from)
        .with_trace_step("build_hexagon_at")?;
    Ok(())
}

fn segment_description(
    segment: &Segment,
    segment_idx: usize,
    snake: &Snake,
    frame_fraction: f32,
    segment_style: SegmentStyle,
    gtx: &GameContext,
) -> SegmentDescription {
    let coming_from = segment.coming_from;
    let going_to = segment_idx
        .checked_sub(1)
        .map(|prev_idx| -snake.body.cells[prev_idx].coming_from)
        .unwrap_or(snake.body.dir);

    let location = segment.pos.to_cartesian(gtx.cell_dim);

    let fraction = match segment_idx {
        // head
        0 => {
            if let SegmentType::BlackHole { just_created: _ } = segment.segment_type {
                // never exceed 0.5 into a black hole, stay there once you get there
                if snake.body.visible_len() == 1 {
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
        i if i == snake.body.visible_len() - 1 && snake.body.grow == 0 => {
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

    let turn_fraction = if segment_idx == 0 {
        snake
            .body
            .turn_start
            .map(|(_, start_frame_fraction)| {
                let max = 1. - start_frame_fraction;

                // when the snake is moving really fast, max == 0 would cause a NaN in the calculation
                if max.abs() < f32::EPSILON {
                    1.
                } else {
                    let covered = frame_fraction - start_frame_fraction;
                    let linear = covered / max;
                    ezing::sine_inout(linear)
                }
            })
            .unwrap_or(1.)
    } else {
        1.
    };

    SegmentDescription {
        destination: location,
        turn: TurnDescription {
            coming_from,
            going_to,
            fraction: turn_fraction,
        },
        fraction,
        draw_style: gtx.prefs.draw_style,
        segment_type: segment.segment_type,
        segment_style,
        z_index: segment.z_index,
        cell_dim: gtx.cell_dim,
    }
}

// TODO: the draw order is actually more complicated
//  heads of non-dying snakes that are going towards
//  the black hole need to be drawn on top of it but
//  those that are going away from the black hole need
//  to be drawn below it (see debug scenario 3)
pub fn snake_mesh(
    snakes: &mut [Snake],
    gtx: &GameContext,
    ctx: &mut Context,
    stats: &mut Stats,
) -> AppResult<Mesh> {
    stats.redrawing_snakes = true;

    let frame_fraction = gtx.frame_stamp.1;

    // Desired total number of subsegments for the whole snake
    // smaller snakes have higher resolution to show more detail
    // (this is intended to work with rainbows)
    const TOTAL_SUBSEGMENTS: usize = 250;

    // Bounds on the number of subsegments per segment to avoid
    // very high numbers of polygons or empty segments
    const MIN_SUBSEGMENTS: usize = 1;
    const MAX_SUBSEGMENTS: usize = 20;

    // TODO (easy): factor out into palette
    let black_hole_color = Color::from_rgb(1, 36, 92);

    // TODO (advanced): make resolution depend on color darkness
    //  (it's easier to discern small differences in blues than in yellows)
    // resolution = solid color subsegments per snake segment
    let color_resolutions: Vec<_> = snakes
        .iter()
        .map(|snake| {
            let resolution = match TOTAL_SUBSEGMENTS / snake.body.visible_len() {
                x if x < MIN_SUBSEGMENTS => MIN_SUBSEGMENTS,
                x if x > MAX_SUBSEGMENTS => MAX_SUBSEGMENTS,
                x => x,
            };

            if resolution > stats.max_color_resolution {
                stats.max_color_resolution = resolution;
            }

            resolution
        })
        .collect();

    let styles: Vec<_> = snakes
        .iter_mut()
        .map(|snake| snake.palette.segment_styles(&snake.body, frame_fraction))
        .collect();

    let mut builder = MeshBuilder::new();

    // The draw order priority list is:
    //  - higher z-index
    //  - black hole
    //  - other

    assert_impl_all!(Snake: Send, Sync);

    let mut descs: Vec<_> = snakes
        .par_iter_mut()
        .zip(styles.into_par_iter())
        .zip(color_resolutions.par_iter())
        .flat_map(|((snake, style), resolution)| {
            snake
                .body
                .cells
                .par_iter()
                .enumerate()
                .zip(style.into_par_iter())
                .map(|((segment_idx, segment), style)| {
                    let desc = segment_description(
                        segment,
                        segment_idx,
                        snake,
                        frame_fraction,
                        style,
                        gtx,
                    );

                    (desc, *resolution)
                })
        })
        .collect();
    descs.par_sort_unstable_by(
        |(desc1, _), (desc2, _)| match desc1.z_index.cmp(&desc2.z_index) {
            Ordering::Equal => {
                if let SegmentType::BlackHole { .. } = desc1.segment_type {
                    Ordering::Greater
                } else if let SegmentType::BlackHole { .. } = desc2.segment_type {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
            ordering => ordering,
        },
    );

    descs.into_iter().try_for_each(|(desc, resolution)| {
        // TODO: animate black hole in
        if let SegmentType::BlackHole { .. } = desc.segment_type {
            let destination = desc.destination + gtx.cell_dim.center();
            let SegmentFraction { start, end } = desc.fraction;
            let real_cell_dim = if (start - end).abs() < f32::EPSILON {
                // snake has died, animate black hole out
                assert!(
                    frame_fraction >= 0.5,
                    "frame fraction ({}) < 0.5",
                    frame_fraction
                );
                let animation_fraction = frame_fraction - 0.5;
                gtx.cell_dim * (1. - animation_fraction)
            } else {
                gtx.cell_dim
            };
            stats.polygons += 1;
            builder.circle(
                DrawMode::fill(),
                destination,
                real_cell_dim.side,
                0.1,
                black_hole_color,
            )?;
        }

        stats.polygons += desc.build(&mut builder, resolution)?;
        Ok::<_, AppError>(())
    })?;

    builder
        .build(ctx)
        .map_err(AppError::from)
        .with_trace_step("snake_mesh")
}
