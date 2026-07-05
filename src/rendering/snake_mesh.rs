use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::error::Result;
use crate::gfx::graphics::{build_circle, Color, DrawMode, Mesh};
use crate::gfx::material::PaletteLut;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnDescription};
use crate::snake::palette::{build_snake_lut, SegmentStyle};
use crate::snake::{Body, Segment, SegmentType, Snake};
use crate::support::partial_min_max::partial_min;

/// Drawable output for all snakes: one shaded mesh + palette LUT per snake, plus
/// an optional plain (default-material) mesh for black-hole circles.
pub struct SnakeRender {
    /// One entry per snake: the segment mesh (with its LUT baked in as texture)
    /// and the LUT itself (kept alive so the texture isn't freed).
    pub shaded: Vec<(Mesh, PaletteLut)>,
    pub black_holes: Option<Mesh>,
}

fn segment_description(
    segment: &Segment,
    segment_idx: usize,
    body: &Body,
    prev_fraction: Option<SegmentFraction>,
    frame_fraction: f32,
    segment_style: SegmentStyle,
    gtx: &GameContext,
) -> SegmentDescription {
    let coming_from = segment.coming_from;
    let going_to = segment.going_to.unwrap_or(body.dir);

    let location = segment.pos.to_cartesian(gtx.cell_dim);

    let fraction = match segment_idx {
        // head
        0 => {
            if let SegmentType::BlackHole { just_created: _ } = segment.segment_type {
                // never exceed 0.5 into a black hole, stay there once you get there
                if body.visible_len() == 1 {
                    // also tail
                    SegmentFraction {
                        start: partial_min(frame_fraction, 0.5).unwrap(),
                        end: 0.5,
                    }
                } else if body.missing_front > 0 {
                    SegmentFraction::appearing(0.5)
                } else {
                    SegmentFraction::appearing(partial_min(frame_fraction, 0.5).unwrap())
                }
            } else {
                SegmentFraction::appearing(frame_fraction)
            }
        }
        // tail
        i if i == body.visible_len() - 1 && body.grow == 0 => {
            if let SegmentType::Eaten { original_food, food_left } = segment.segment_type {
                let frac = ((original_food - food_left) as f32 + frame_fraction) / (original_food + 1) as f32;
                SegmentFraction::disappearing(frac)
            } else {
                SegmentFraction::disappearing(frame_fraction)
            }
        }
        // body
        _ => SegmentFraction::solid(),
    };

    let turn_fraction = if segment_idx == 0 {
        body.turn_start
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
        segment_idx,
        destination: location,
        turn: TurnDescription {
            coming_from,
            going_to,
            fraction: turn_fraction,
        },
        prev_fraction,
        fraction,
        draw_style: gtx.prefs.draw_style,
        segment_type: segment.segment_type,
        segment_style,
        z_index: segment.z_index,
        cell_dim: gtx.cell_dim,
    }
}

/// Build the drawable meshes for every snake. Each snake becomes one polygon per
/// segment (no color subdivision); color is applied per-pixel by the snake
/// shader sampling that snake's palette LUT. Black-hole circles are collected
/// separately and drawn on the default material.
pub fn snake_mesh(snakes: &mut [Snake], gtx: &GameContext, ftx: &FpsContext, stats: &mut Stats) -> Result<SnakeRender> {
    stats.redrawing_snakes = true;

    let frame_fraction = ftx.last_graphics_update.1;

    // TODO (easy): factor out into palette
    let black_hole_color = Color::from_rgb(1, 36, 92);

    let mut black_hole_parts: Vec<Mesh> = vec![];
    let mut shaded = Vec::with_capacity(snakes.len());

    for snake in snakes.iter_mut() {
        let body = &snake.body;
        let num_segments = body.segments.len();

        // Per-snake palette LUT (each segment its own fixed-size slot).
        let styles: Vec<SegmentStyle> = snake.palette.segment_styles(body, frame_fraction).collect();
        let lut_colors = build_snake_lut(&styles);
        let lut_size = lut_colors.len();
        let lut = PaletteLut::new(&lut_colors);

        // Per-segment descriptions (head → tail).
        let mut prev_fraction = None;
        let descs: Vec<SegmentDescription> = body
            .segments
            .iter()
            .enumerate()
            .zip(styles)
            .map(|((segment_idx, segment), style)| {
                let desc = segment_description(segment, segment_idx, body, prev_fraction, frame_fraction, style, gtx);
                prev_fraction = Some(desc.fraction);
                desc
            })
            .collect();

        // Black-hole circles (default material), drawn separately.
        for desc in &descs {
            if let SegmentType::BlackHole { .. } = desc.segment_type {
                let destination = desc.destination + gtx.cell_dim.center();
                let SegmentFraction { start, end } = desc.fraction;
                let real_cell_dim = if (start - end).abs() < f32::EPSILON {
                    // snake has died, animate black hole out
                    let animation_fraction = frame_fraction - 0.5;
                    gtx.cell_dim * (1. - animation_fraction)
                } else {
                    gtx.cell_dim
                };
                black_hole_parts.push(build_circle(DrawMode::fill(), destination, real_cell_dim.side, black_hole_color));
                stats.polygons += 1;
            }
        }

        // Shaded segments. Draw tail → head so the head paints on top.
        let segments = descs.iter().rev().map(|desc| {
            stats.polygons += 1;
            desc.build_shaded(num_segments, lut_size)
        });
        let mut mesh = Mesh::combine(segments);
        mesh.set_texture(lut.texture());
        shaded.push((mesh, lut));
    }

    let black_holes = (!black_hole_parts.is_empty()).then(|| Mesh::combine(black_hole_parts));

    Ok(SnakeRender { shaded, black_holes })
}
