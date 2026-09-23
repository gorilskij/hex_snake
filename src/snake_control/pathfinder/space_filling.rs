use std::cmp::Reverse;
use std::collections::HashSet;

use super::{surroundings, Committed, Goals, Leg, Obstacles, Path, PathFinder, Start};
use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexDim, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::view::snakes::Snakes;

/// How far ahead the crawl plans. It is walked rather than recomputed, so it
/// only needs to be long enough not to be redone all the time; every step
/// costs a flood fill per neighbour, and a long route goes stale anyway.
const CRAWL_LENGTH: usize = 20;

/// Survival fallback for when no target can be reached: a path that keeps
/// stepping into the free neighboring cell with the most room behind it,
/// preferring the gentlest turn, and never back onto itself.
pub struct SpaceFilling;

impl PathFinder for SpaceFilling {
    fn get_path(
        &self,
        start: Start,
        _goals: &Goals,
        // the crawl is only ever planned on its own, so there is never a
        // committed route to avoid
        _committed: Committed,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Leg> {
        let obstacles = Obstacles::new(body, knowledge, other_snakes);
        let cells = crawl(start, gtx.board_dim, |pos| obstacles.blocks(pos));
        // going nowhere in particular: no target
        (cells.len() >= 2).then_some(Leg { cells, target: None })
    }
}

/// The crawl from `start`, `start` included, as far as [`CRAWL_LENGTH`] or
/// until it has nowhere left to go.
fn crawl(start: Start, board_dim: HexDim, blocks: impl Fn(HexPoint) -> bool) -> Path {
    let (mut here, mut heading) = start;
    let mut cells = Path::from([here]);
    let mut taken = HashSet::from([here]);

    while cells.len() <= CRAWL_LENGTH {
        let blocked = |pos| blocks(pos) || taken.contains(&pos);
        let next = Dir::iter()
            .filter(|&dir| dir != -heading)
            .map(|dir| (dir, here.wrapping_translate(dir, 1, board_dim)))
            .filter(|&(_, pos)| !blocked(pos))
            .max_by_key(|&(dir, pos)| {
                let turn = heading.clockwise_distance_to(dir);
                let room = surroundings(pos, board_dim, blocked).0.len();
                (room, Reverse(turn.min(6 - turn)))
            });
        let Some((dir, pos)) = next else { break };
        cells.push_back(pos);
        taken.insert(pos);
        (here, heading) = (pos, dir);
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOARD: HexDim = HexDim { h: 20, v: 20 };

    fn steps_connect(cells: &Path) -> bool {
        cells
            .iter()
            .zip(cells.iter().skip(1))
            .all(|(a, b)| a.single_step_dir_to(*b, BOARD).is_some())
    }

    #[test]
    fn the_crawl_is_a_path_that_never_crosses_itself() {
        let start = (HexPoint { h: 10, v: 10 }, Dir::U);
        let cells = crawl(start, BOARD, |_| false);

        assert_eq!(cells.len(), CRAWL_LENGTH + 1);
        assert!(steps_connect(&cells), "{cells:?}");
        assert_eq!(cells.iter().collect::<HashSet<_>>().len(), cells.len(), "{cells:?}");
    }

    #[test]
    fn the_crawl_stays_out_of_the_way_and_stops_when_boxed_in() {
        // a pocket of 5 cells above the start, walled in
        let pocket: HashSet<HexPoint> = (5..=10).map(|v| HexPoint { h: 10, v }).collect();
        let start = (HexPoint { h: 10, v: 10 }, Dir::U);
        let cells = crawl(start, BOARD, |pos| !pocket.contains(&pos));

        assert_eq!(cells.len(), pocket.len(), "it fills the pocket: {cells:?}");
        assert!(cells.iter().all(|pos| pocket.contains(pos)), "{cells:?}");
    }
}
