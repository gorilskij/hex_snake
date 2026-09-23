use anyhow::Result;

use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::rendering;
use crate::rendering::segments::cap::build_round_caps;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnDescription};
use crate::rendering::segments::marks::{Joins, build_marks};
use crate::snake::palette::{SegmentStyle, build_snake_lut};
use crate::snake::{Body, Segment, Snake, ZIndex};
use crate::support::material::PaletteLut;
use crate::support::mesh::Mesh;

/// Drawable output for all snakes, as pieces to draw in order.
///
/// A snake is one piece unless it crosses over or under something: then the
/// segments on either side of each change of `z_index` are separate pieces,
/// so that every snake's pieces can be drawn by z, together. That is what lets
/// a snake be over another at one crossing and under it at the next.
pub struct SnakeRender {
    /// Shaded meshes, in the order to draw them. Each holds its snake's LUT
    /// as texture, which keeps it alive (`Texture2D` is reference counted).
    pub pieces: Vec<Mesh>,
}

/// Describe every segment of a body, head → tail. Shared with the collision
/// centerline so both read the same geometry off the same `Body`.
pub fn segment_descriptions(body: &Body, gtx: &GameContext) -> Vec<SegmentDescription> {
    body.segments
        .iter()
        .enumerate()
        .map(|(segment_idx, segment)| segment_description(segment, segment_idx, body, gtx))
        .collect()
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
pub fn snake_mesh(snakes: &mut [Snake], apples: &[Apple], gtx: &GameContext) -> Result<SnakeRender> {
    let mut pieces = vec![];

    for snake in snakes.iter_mut() {
        let body = &snake.body;
        let num_segments = body.segments.len();

        // Per-snake palette LUT (each segment its own fixed-size slot).
        let styles: Vec<SegmentStyle> = snake.palette.segment_styles(body).collect();
        let lut_colors = build_snake_lut(&styles);
        let lut_size = lut_colors.len();
        let lut = PaletteLut::new(&lut_colors);

        // Per-segment descriptions (head → tail).
        let mut descs = segment_descriptions(body, gtx);

        // Round end caps (smooth style): truncate the body ribbon by one cap
        // radius at each end and fill with half-circle caps.
        let (tail_cap, head_cap) = if gtx.prefs.draw_style == rendering::Style::Smooth && !descs.is_empty() {
            build_round_caps(&mut descs, num_segments, lut_size)
        } else {
            (None, None)
        };

        // Which segments carry passability marks (head → tail), and whether the
        // head's marks already join the next head segment's.
        let eat_mechanics = snake.eat_mechanics;
        let marked: Vec<bool> = body
            .segments
            .iter()
            .map(|segment| eat_mechanics.is_marked(segment.segment_type))
            .collect();
        let next_marked = snake
            .upcoming_eaten_segment(apples, gtx)
            .is_some_and(|segment_type| eat_mechanics.is_marked(segment_type));

        // Shaded segments, each with its z. Draw tail → head so the head
        // paints on top; the caps keep that order (tail cap under, head cap
        // over) and their end's z. A segment's marks go directly on top of it.
        let tail_z = descs.last().map_or(0, |desc| desc.z_index);
        let head_z = descs.first().map_or(0, |desc| desc.z_index);
        let parts = tail_cap
            .map(|cap| (tail_z, cap))
            .into_iter()
            .chain(descs.iter().rev().flat_map(|desc| {
                let idx = desc.segment_idx;
                let marks = marked[idx].then(|| {
                    let joins = Joins {
                        tail: marked.get(idx + 1) == Some(&true),
                        head: if idx == 0 { next_marked } else { marked[idx - 1] },
                    };
                    build_marks(desc, joins, num_segments, lut_size)
                });
                std::iter::once(desc.build_shaded(num_segments, lut_size))
                    .chain(marks)
                    .map(|mesh| (desc.z_index, mesh))
            }))
            .chain(head_cap.map(|cap| (head_z, cap)));
        for (z, run) in runs(parts) {
            let mut mesh = Mesh::combine(run);
            mesh.set_texture(lut.texture());
            pieces.push((z, mesh));
        }
    }

    Ok(SnakeRender { pieces: draw_order(pieces) })
}

/// Consecutive items with the same z, grouped, in order.
fn runs<T>(items: impl IntoIterator<Item = (ZIndex, T)>) -> Vec<(ZIndex, Vec<T>)> {
    let mut runs: Vec<(ZIndex, Vec<T>)> = vec![];
    for (z, item) in items {
        match runs.last_mut() {
            Some((run_z, run)) if *run_z == z => run.push(item),
            _ => runs.push((z, vec![item])),
        }
    }
    runs
}

/// Lowest z first. Equal z keeps the order given (snake by snake, tail to
/// head), which is how pieces that don't cross anything have always drawn.
fn draw_order<T>(mut pieces: Vec<(ZIndex, T)>) -> Vec<T> {
    pieces.sort_by_key(|&(z, _)| z);
    pieces.into_iter().map(|(_, piece)| piece).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snake_is_cut_only_where_its_z_changes() {
        let parts = [(0, 'a'), (0, 'b'), (1, 'c'), (1, 'd'), (0, 'e'), (-1, 'f'), (0, 'g')];
        assert_eq!(
            runs(parts),
            [
                (0, vec!['a', 'b']),
                (1, vec!['c', 'd']),
                (0, vec!['e']),
                (-1, vec!['f']),
                (0, vec!['g']),
            ]
        );
        assert_eq!(
            runs([(0, 'a'), (0, 'b')]),
            [(0, vec!['a', 'b'])],
            "one piece without crossings"
        );
    }

    /// A over B at one crossing (its +1 piece) and under B at another (its -1
    /// piece): B goes between them.
    #[test]
    fn a_snake_can_be_over_and_under_another_at_once() {
        let pieces = vec![
            (0, "A1"),
            (1, "A over"),
            (0, "A2"),
            (-1, "A under"),
            (0, "A3"),
            (0, "B"),
        ];
        assert_eq!(draw_order(pieces), ["A under", "A1", "A2", "A3", "B", "A over"]);
    }

    /// Each snake over the other at a different crossing.
    #[test]
    fn two_snakes_can_each_be_over_the_other() {
        let pieces = vec![(0, "A1"), (1, "A over"), (0, "A2"), (0, "B1"), (1, "B over"), (0, "B2")];
        assert_eq!(draw_order(pieces), ["A1", "A2", "B1", "B2", "A over", "B over"]);
    }
}
