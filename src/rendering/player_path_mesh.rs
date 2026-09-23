use anyhow::Result;
use macroquad::color::Color;

use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::{Dir, HexPoint};
use crate::rendering::segments::centerline::centerline_polyline;
use crate::rendering::segments::descriptions::{SegmentDescription, SegmentFraction, TurnDescription};
use crate::rendering::Style;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::{SegmentType, Snake};
use crate::support::mesh::{build_line, Mesh};
use crate::view::snakes::OtherSnakes;

/// How wide the path is drawn, as a fraction of a cell's side.
const PATH_WIDTH: f32 = 2. / 5.;

/// How finely a turn's arc is sampled. Straight stretches need no subdivision.
const ARC_STEPS: usize = 12;

/// How bright the route is drawn, stepping from the leg the snake is on to the
/// last one, so the nearest target always reads as the most immediate.
const FIRST_LEG_BRIGHTNESS: f32 = 0.7;
const LAST_LEG_BRIGHTNESS: f32 = 0.3;

/// The brightness of leg `idx` of `legs`: a linear step between the two, and
/// the first leg's own brightness when there is only one.
fn leg_color(idx: usize, legs: usize) -> Color {
    let brightness = if legs < 2 {
        FIRST_LEG_BRIGHTNESS
    } else {
        let step = (LAST_LEG_BRIGHTNESS - FIRST_LEG_BRIGHTNESS) / (legs - 1) as f32;
        FIRST_LEG_BRIGHTNESS + step * idx as f32
    };
    Color::new(brightness, brightness, brightness, 1.)
}

/// The autopilot's path, as two meshes: the part to draw under the snakes,
/// and the part over the player's own eaten segments, which the path passes
/// through and which would otherwise hide it.
pub fn player_path_mesh(
    player_snake: &mut Snake,
    other_snakes: OtherSnakes,
    apples: &[Apple],
    gtx: &GameContext,
) -> Option<Result<(Mesh, Mesh)>> {
    let autopilot = player_snake.autopilot.as_mut()?;
    let knowledge = Knowledge::accurate(&player_snake.eat_mechanics);
    let plan = autopilot.get_plan(&player_snake.body, Some(&knowledge), &other_snakes, apples, gtx)?;

    // the head is where the route starts, so it stays over the route even when
    // eaten; only the body the route crosses further on is drawn under it
    let eaten: Vec<_> = player_snake
        .body
        .segments
        .iter()
        .skip(1)
        .filter(|segment| matches!(segment.segment_type, SegmentType::Eaten { .. }))
        .map(|segment| segment.pos)
        .collect();

    let mut under: Vec<Mesh> = vec![];
    let mut over: Vec<Mesh> = vec![];

    // where the route enters its first cell: whichever way the head came in
    let mut coming_from = player_snake.body.segments[0].coming_from;
    // and how far into that cell the head has got, which is where the line
    // starts: exactly at the head's tip, so the snake eats the line
    // continuously instead of a whole cell of it going at once
    let head_fraction = player_snake.body.head_fraction;
    let width = gtx.cell_dim.side * PATH_WIDTH;

    // Legs share the cell they meet in, so they are walked as one sequence of
    // cells; each cell is drawn in its own leg's color, giving a step gradient
    // from the leg the snake is on to the last one. A shared cell is a target,
    // and belongs to both legs — it is marked so it can be drawn as one half of
    // each.
    let legs = plan.legs.len();
    let cells: Vec<(HexPoint, usize, bool)> = plan
        .legs
        .iter()
        .enumerate()
        .flat_map(|(leg_idx, leg)| {
            let shared = if leg_idx == 0 { 0 } else { 1 };
            let target = leg_idx + 1 < legs;
            leg.cells
                .iter()
                .enumerate()
                .skip(shared)
                .map(move |(idx, &pos)| (pos, leg_idx, target && idx + 1 == leg.cells.len()))
        })
        .collect();
    let last = cells.len().checked_sub(1)?;

    for (idx, &(pos, leg_idx, is_target)) in cells.iter().enumerate() {
        // the last cell has nowhere to go, so the line leaves it straight
        let going_to = cells
            .get(idx + 1)
            .and_then(|&(next_pos, ..)| pos.single_step_dir_to(next_pos, gtx.board_dim))
            .unwrap_or(-coming_from);

        // the line stops at the last target's center, and starts at the head
        let end = if idx == last { 0.5 } else { 1. };
        let start = if idx == 0 { head_fraction.min(end) } else { 0. };

        // The route curves through each cell exactly as a snake taking it
        // would: the same centerline the ribbon is built around, for the same
        // turn. Cells are drawn one at a time, which is also why a teleport
        // needs no special case — each half of the wrap is drawn where it is.
        //
        // A target's cell is the exception. There the route is drawn as two
        // straight halves meeting at the center, each in its own leg's color:
        // the apple covers the join, so a curve through it would only bend the
        // line away from the cell it is about to be eaten in.
        let pieces: [(Dir, Dir, SegmentFraction, usize); 2] = if is_target {
            [
                (coming_from, -coming_from, SegmentFraction { start, end: 0.5 }, leg_idx),
                // `start` matters here too: the head can be past the center
                (-going_to, going_to, SegmentFraction { start: start.max(0.5), end }, leg_idx + 1),
            ]
        } else {
            [
                (coming_from, going_to, SegmentFraction { start, end }, leg_idx),
                // one piece is enough; an empty fraction draws nothing
                (coming_from, going_to, SegmentFraction { start: 0., end: 0. }, leg_idx),
            ]
        };

        for (coming_from, going_to, fraction, leg_idx) in pieces {
            if going_to == coming_from || fraction.end - fraction.start <= 0. {
                continue;
            }

            let desc = SegmentDescription {
                segment_idx: 0,
                destination: pos.to_cartesian(gtx.cell_dim),
                turn: TurnDescription { coming_from, going_to, fraction: 1. },
                fraction,
                // only the turn decides the centerline, but the route is a
                // smooth curve whatever the snakes are drawn as
                draw_style: Style::Smooth,
                segment_type: SegmentType::Normal,
                z_index: 0,
                cell_dim: gtx.cell_dim,
            };

            // round caps and shared end points join the cells into one line
            let points = centerline_polyline(&desc, desc.fraction, ARC_STEPS);
            let parts = if eaten.contains(&pos) { &mut over } else { &mut under };
            parts.push(build_line(&points, width, leg_color(leg_idx, legs)));
        }
        coming_from = -going_to;
    }

    Some(Ok((Mesh::combine(under), Mesh::combine(over))))
}
