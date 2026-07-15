use anyhow::Result;
use macroquad::color::Color;

use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::rendering;
use crate::rendering::segments::cap::build_round_caps;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnDescription};
use crate::snake::palette::{build_snake_lut, SegmentStyle};
use crate::snake::{Body, Segment, SegmentType, Snake};
use crate::support::material::PaletteLut;
use crate::support::mesh::{build_circle, DrawMode, Mesh};
use crate::support::partial_min_max::partial_min;

/// Drawable output for all snakes: one shaded mesh + palette LUT per snake, plus
/// an optional plain (default-material) mesh for black-hole circles.
pub struct SnakeRender {
    /// One entry per snake: the segment mesh (with its LUT baked in as texture)
    /// and the LUT itself (kept alive so the texture isn't freed).
    pub shaded: Vec<(Mesh, PaletteLut)>,
    pub black_holes: Option<Mesh>,
}

fn segment_description(segment: &Segment, segment_idx: usize, body: &Body, gtx: &GameContext) -> SegmentDescription {
    let coming_from = segment.coming_from;
    let going_to = segment.going_to.unwrap_or(body.dir);

    let location = segment.pos.to_cartesian(gtx.cell_dim);

    let fraction = match segment_idx {
        // head
        0 => {
            if let SegmentType::BlackHole { .. } = segment.segment_type {
                // never exceed 0.5 into a black hole, stay there once you get there
                if body.visible_len() == 1 {
                    // also tail
                    SegmentFraction {
                        start: partial_min(body.segment_fraction, 0.5).unwrap(),
                        end: 0.5,
                    }
                } else if body.missing_front > 0 {
                    SegmentFraction::appearing(0.5)
                } else {
                    SegmentFraction::appearing(partial_min(body.segment_fraction, 0.5).unwrap())
                }
            } else {
                SegmentFraction::appearing(body.segment_fraction)
            }
        }
        // tail
        i if i == body.visible_len() - 1 && body.grow == 0 => {
            if let SegmentType::Eaten { original_food, food_left } = segment.segment_type {
                let frac = ((original_food - food_left) as f32 + body.segment_fraction) / (original_food + 1) as f32;
                SegmentFraction::disappearing(frac)
            } else {
                SegmentFraction::disappearing(body.segment_fraction)
            }
        }
        // body
        _ => SegmentFraction::solid(),
    };

    let turn_fraction = if segment_idx == 0 {
        body.turn_start
            .map(|start_fraction| {
                let max = 1. - start_fraction;

                // when the snake is moving really fast, max == 0 would cause a NaN in the calculation
                if max.abs() < f32::EPSILON {
                    1.
                } else {
                    let covered = body.segment_fraction - start_fraction;
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
        fraction,
        draw_style: gtx.prefs.draw_style,
        segment_type: segment.segment_type,
        z_index: segment.z_index,
        cell_dim: gtx.cell_dim,
    }
}

/// Build the drawable meshes for every snake. Each snake becomes one polygon per
/// segment (no color subdivision); color is applied per-pixel by the snake
/// shader sampling that snake's palette LUT. Black-hole circles are collected
/// separately and drawn on the default material.
pub fn snake_mesh(snakes: &mut [Snake], gtx: &GameContext, stats: &mut Stats) -> Result<SnakeRender> {
    stats.redrawing_snakes = true;

    // TODO (easy): factor out into palette
    let black_hole_color = Color::from_rgba(1, 36, 92, 255);

    let mut black_hole_parts: Vec<Mesh> = vec![];
    let mut shaded = Vec::with_capacity(snakes.len());

    for snake in snakes.iter_mut() {
        let body = &snake.body;
        let num_segments = body.segments.len();

        // Per-snake palette LUT (each segment its own fixed-size slot).
        let styles: Vec<SegmentStyle> = snake.palette.segment_styles(body).collect();
        let lut_colors = build_snake_lut(&styles);
        let lut_size = lut_colors.len();
        let lut = PaletteLut::new(&lut_colors);

        // Per-segment descriptions (head → tail).
        let mut descs: Vec<SegmentDescription> = body
            .segments
            .iter()
            .enumerate()
            .map(|(segment_idx, segment)| segment_description(segment, segment_idx, body, gtx))
            .collect();

        // Black-hole circles (default material), drawn separately.
        for desc in &descs {
            if let SegmentType::BlackHole { .. } = desc.segment_type {
                let destination = desc.destination + gtx.cell_dim.center();
                let SegmentFraction { start, end } = desc.fraction;
                let real_cell_dim = if (start - end).abs() < f32::EPSILON {
                    // snake has died, animate black hole out
                    let animation_fraction = body.segment_fraction - 0.5;
                    gtx.cell_dim * (1. - animation_fraction)
                } else {
                    gtx.cell_dim
                };
                black_hole_parts.push(build_circle(
                    DrawMode::fill(),
                    destination,
                    real_cell_dim.side,
                    black_hole_color,
                ));
                stats.polygons += 1;
            }
        }

        // Round end caps (smooth style): truncate the body ribbon by one cap
        // radius at each end and fill with half-circle caps.
        let (tail_cap, head_cap) = if gtx.prefs.draw_style == rendering::Style::Smooth && !descs.is_empty() {
            let caps = build_round_caps(&mut descs, num_segments, lut_size);
            stats.polygons += (caps.0.is_some() as usize) + (caps.1.is_some() as usize);
            caps
        } else {
            (None, None)
        };

        // Shaded segments. Draw tail → head so the head paints on top; the
        // caps keep that order (tail cap under, head cap over).
        let segments = tail_cap
            .into_iter()
            .chain(descs.iter().rev().map(|desc| {
                stats.polygons += 1;
                desc.build_shaded(num_segments, lut_size)
            }))
            .chain(head_cap);
        let mut mesh = Mesh::combine(segments);
        mesh.set_texture(lut.texture());
        shaded.push((mesh, lut));
    }

    let black_holes = (!black_hole_parts.is_empty()).then(|| Mesh::combine(black_hole_parts));

    Ok(SnakeRender { shaded, black_holes })
}
