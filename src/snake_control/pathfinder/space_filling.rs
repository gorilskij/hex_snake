use std::cmp::Reverse;
use std::collections::HashSet;

use super::{Committed, Leg, Obstacles, Path, PathFinder, Start};
use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexDim, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;

/// Survival fallback for when no target can be reached: a one-step path into
/// the free neighboring cell with the most room behind it, preferring the
/// gentlest turn.
pub struct SpaceFilling;

impl PathFinder for SpaceFilling {
    fn get_path(
        &self,
        start: Start,
        _targets: &dyn Targets,
        // the crawl is one step long and only ever planned on its own, so
        // there is never a committed route to avoid
        _committed: Committed,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Leg> {
        let obstacles = Obstacles::new(body, knowledge, other_snakes);
        let (from, heading) = start;

        Dir::iter()
            .filter(|&dir| dir != -heading)
            .map(|dir| (dir, from.wrapping_translate(dir, 1, gtx.board_dim)))
            .filter(|&(_, pos)| !obstacles.blocks(pos))
            .max_by_key(|&(dir, pos)| {
                let turn = heading.clockwise_distance_to(dir);
                (room(pos, &obstacles, gtx.board_dim), Reverse(turn.min(6 - turn)))
            })
            // going nowhere in particular: no target
            .map(|(_, pos)| Leg {
                cells: Path::from([from, pos]),
                target: None,
            })
    }
}

/// How many cells can be reached from `start` without running into anything.
fn room(start: HexPoint, obstacles: &Obstacles, board_dim: HexDim) -> usize {
    let mut seen = HashSet::from([start]);
    let mut stack = vec![start];
    while let Some(pos) = stack.pop() {
        for dir in Dir::iter() {
            let next = pos.wrapping_translate(dir, 1, board_dim);
            if !obstacles.blocks(next) && seen.insert(next) {
                stack.push(next);
            }
        }
    }
    seen.len()
}
