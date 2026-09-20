use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use super::{Committed, Leg, Obstacle, Obstacles, Path, PathFinder, Start};
use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;

/// The cost of each thing a path does. The search finds the cheapest path to
/// any target.
#[derive(Copy, Clone, Debug)]
pub struct Weights {
    /// Moving one cell.
    pub step: u32,
    /// Turning by 60°.
    pub blunt_turn: u32,
    /// Turning by 120°.
    pub sharp_turn: u32,
    /// Wrapping around the edge of the board.
    pub teleport: u32,
    /// Entering a cell occupied by a segment the snake can pass through.
    pub pass_through: u32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            step: 1,
            blunt_turn: 2,
            sharp_turn: 4,
            teleport: 15,
            pass_through: 0,
        }
    }
}

impl Weights {
    fn turn(&self, from: Dir, to: Dir) -> u32 {
        match from.clockwise_distance_to(to) {
            0 => 0,
            1 | 5 => self.blunt_turn,
            _ => self.sharp_turn,
        }
    }
}

/// A cell together with the direction the head entered it in: turning costs
/// depend on where the head came from, so the same cell reached along two
/// headings are different states.
type State = (HexPoint, Dir);

/// Cheapest path to the nearest target (Dijkstra over [`State`]s).
///
/// TODO: this is Dijkstra, i.e. A* with `h = 0`, and it explores in every
///  direction. A heuristic would cut that down, and the state and costs are
///  already the right shape for one — the queue would order by `cost + h`
///  instead of `cost`. Note that a closed-form hex distance is *not* usable
///  here: `wrap_around` is not a lattice translation (the diagonal axes wrap
///  each line onto itself, with a length that varies by position), so a
///  wrap-aware distance has no simple formula. The workable form is a landmark
///  heuristic: run this search backwards from each target once, and read the
///  exact remaining cost out of that field. Admissible by construction, and it
///  handles wrapping without any distance math.
pub struct WeightedBFS {
    pub weights: Weights,
}

impl PathFinder for WeightedBFS {
    fn get_path(
        &self,
        start: Start,
        targets: &dyn Targets,
        committed: Committed,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Leg> {
        let weights = &self.weights;
        let obstacles = Obstacles::new(body, knowledge, other_snakes);
        // a target an earlier leg is already going for is no longer one
        let targets: HashSet<_> = targets
            .iter()
            .filter(|pos| !committed.targets.contains(pos))
            .collect();
        // where the route has already been is in the way of where it goes next
        let committed: HashMap<_, _> = committed.cells.iter().copied().collect();

        let (from, _) = start;
        let mut costs = HashMap::from([(start, 0)]);
        let mut parents = HashMap::new();
        let mut queue = BinaryHeap::from([Reverse((0, start))]);

        while let Some(Reverse((cost, state))) = queue.pop() {
            let (pos, dir) = state;
            if cost > costs[&state] {
                // superseded by a cheaper way here
                continue;
            }
            if pos != from && targets.contains(&pos) {
                return Some(Leg {
                    cells: path_to(state, &parents),
                    target: Some(pos),
                });
            }

            // a snake can never reverse
            for new_dir in Dir::iter().filter(|&new_dir| new_dir != -dir) {
                let (new_pos, teleported) = pos.explicit_wrapping_translate(new_dir, 1, gtx.board_dim);
                // the worse of what is there now and what the plan puts there
                let obstacle = obstacles.at(new_pos).max(committed.get(&new_pos).copied());
                if obstacle == Some(Obstacle::Blocked) {
                    continue;
                }

                let new_cost = cost
                    + weights.step
                    + weights.turn(dir, new_dir)
                    + if teleported { weights.teleport } else { 0 }
                    + if obstacle == Some(Obstacle::Passable) {
                        weights.pass_through
                    } else {
                        0
                    };

                let new_state = (new_pos, new_dir);
                if costs.get(&new_state).is_none_or(|&old_cost| new_cost < old_cost) {
                    costs.insert(new_state, new_cost);
                    parents.insert(new_state, state);
                    queue.push(Reverse((new_cost, new_state)));
                }
            }
        }

        None
    }
}

fn path_to(mut state: State, parents: &HashMap<State, State>) -> Path {
    let mut path = VecDeque::from([state.0]);
    while let Some(&parent) = parents.get(&state) {
        path.push_front(parent.0);
        state = parent;
    }
    path
}
