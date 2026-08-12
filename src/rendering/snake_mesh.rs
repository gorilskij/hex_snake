use anyhow::Result;

use crate::app::game_context::GameContext;
use crate::app::stats::Stats;
use crate::rendering;
use crate::rendering::segments::cap::build_round_caps;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnDescription};
use crate::snake::palette::{build_snake_lut, SegmentStyle};
use crate::snake::{Body, Segment, Snake};
use crate::support::material::PaletteLut;
use crate::support::mesh::Mesh;


/// Drawable output for all snakes: one shaded mesh + palette LUT per snake.
pub struct SnakeRender {
    /// One entry per snake: the segment mesh (with its LUT baked in as texture)
    /// and the LUT itself (kept alive so the texture isn't freed).
    pub shaded: Vec<(Mesh, PaletteLut)>,
}

fn segment_description(segment: &Segment, segment_idx: usize, body: &Body, gtx: &GameContext) -> SegmentDescription {
    let coming_from = segment.coming_from;
    let going_to = segment.going_to.unwrap_or(body.dir);

    let location = segment.pos.to_cartesian(gtx.cell_dim);

    // The body spans from the tail's recede to the head's progress; a snake
    // down to a single segment is both at once. Both ends are pinned by the
    // model when they are in a hole, so nothing needs clamping here.
    let fraction = SegmentFraction {
        start: if segment_idx == body.visible_len() - 1 {
            body.tail_fraction()
        } else {
            0.
        },
        end: if segment_idx == 0 { body.head_fraction } else { 1. },
    };

    let turn_fraction = if segment_idx == 0 {
        body.turn_start
            .map(|start_fraction| {
                let max = 1. - start_fraction;

                // when the snake is moving really fast, max == 0 would cause a NaN in the calculation
                if max.abs() < f32::EPSILON {
                    1.
                } else {
                    let covered = body.head_fraction - start_fraction;
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
/// shader sampling that snake's palette LUT.
pub fn snake_mesh(snakes: &mut [Snake], gtx: &GameContext, stats: &mut Stats) -> Result<SnakeRender> {
    stats.redrawing_snakes = true;

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

    Ok(SnakeRender { shaded })
}
