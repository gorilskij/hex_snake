use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use super::{Obstacle, Obstacles, Path, PathFinder};
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
            blunt_turn: 5,
            sharp_turn: 5,
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
pub struct WeightedBFS {
    pub weights: Weights,
}

impl PathFinder for WeightedBFS {
    fn get_path(
        &self,
        targets: &dyn Targets,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Path> {
        let weights = &self.weights;
        let obstacles = Obstacles::new(body, knowledge, other_snakes);
        let targets: HashSet<_> = targets.iter().collect();

        let head = body.segments[0].pos;
        let start = (head, body.dir);
        let mut costs = HashMap::from([(start, 0)]);
        let mut parents = HashMap::new();
        let mut queue = BinaryHeap::from([Reverse((0, start))]);

        while let Some(Reverse((cost, state))) = queue.pop() {
            let (pos, dir) = state;
            if cost > costs[&state] {
                // superseded by a cheaper way here
                continue;
            }
            if pos != head && targets.contains(&pos) {
                return Some(path_to(state, &parents));
            }

            // a snake can never reverse
            for new_dir in Dir::iter().filter(|&new_dir| new_dir != -dir) {
                let (new_pos, teleported) = pos.explicit_wrapping_translate(new_dir, 1, gtx.board_dim);
                let obstacle = obstacles.at(new_pos);
                if obstacle == Some(Obstacle::Blocked) {
                    continue;
                }

                let new_cost = cost
                    + weights.step
                    + weights.turn(dir, new_dir)
                    + if teleported { weights.teleport } else { 0 }
                    + if obstacle == Some(Obstacle::Passable) { weights.pass_through } else { 0 };

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
