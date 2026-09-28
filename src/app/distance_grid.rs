use std::collections::{HashMap, HashSet};
use std::mem;

use itertools::Itertools;

use crate::basic::{Dir, HexDim, HexPoint};
use crate::snake::Snake;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake_control::pathfinder::Obstacles;
use crate::view::snakes::Snakes;

pub type Distance = f32;
type GridData = HashMap<HexPoint, Distance>;

struct Iter {
    board_dim: HexDim,

    // -- bfs --
    seen: HashSet<HexPoint>,
    obstacles: Obstacles,
    // all the positions in a generation have the same distance
    dist: usize,
    // search will only continue from generation_alive
    generation_alive: Vec<HexPoint>,
    generation_dead: Vec<HexPoint>,
    // once a new generation is computed, it's iterated over to
    // return the values one by one
    output_idx: usize,
}

impl Iterator for Iter {
    type Item = (HexPoint, Distance);

    fn next(&mut self) -> Option<Self::Item> {
        let num_alive = self.generation_alive.len();
        let num_dead = self.generation_dead.len();

        if self.output_idx < num_alive {
            let ret = self.generation_alive[self.output_idx];
            self.output_idx += 1;
            Some((ret, self.dist as Distance))
        } else if self.output_idx < num_alive + num_dead {
            let ret = self.generation_dead[self.output_idx - num_alive];
            self.output_idx += 1;
            Some((ret, self.dist as Distance))
        } else {
            // bfs step
            let board_dim = self.board_dim;

            self.generation_dead = vec![];
            let generation_alive = mem::take(&mut self.generation_alive);

            generation_alive
                .into_iter()
                .flat_map(move |pos| Dir::iter().map(move |dir| pos.wrapping_translate(dir, 1, board_dim)))
                .filter(|new_pos| !self.seen.contains(new_pos))
                .sorted_unstable()
                .dedup()
                .for_each(|pos| {
                    if self.obstacles.blocks(pos) {
                        self.generation_dead.push(pos)
                    } else {
                        self.generation_alive.push(pos)
                    }
                });

            if self.generation_alive.is_empty() {
                None
            } else {
                self.seen.extend(&self.generation_alive);
                self.seen.extend(&self.generation_dead);
                self.dist += 1;
                self.output_idx = 1;
                Some((self.generation_alive[0], self.dist as Distance))
            }
        }
    }
}

fn find_distances(player_snake: &Snake, other_snakes: impl Snakes, board_dim: HexDim) -> GridData {
    let knowledge = Knowledge::accurate(&player_snake.eat_mechanics);
    let obstacles = Obstacles::new(&player_snake.body, Some(&knowledge), &other_snakes);

    // setup bfs; the head's own cell is never given a distance, and marking
    // it seen keeps the search from finding it again from its neighbours
    let head = player_snake.head().pos;
    Iter {
        board_dim,
        seen: HashSet::from([head]),
        obstacles,
        dist: 0,
        generation_alive: vec![head],
        generation_dead: vec![],
        output_idx: 1, // trigger bfs step immediately
    }
    .collect()
}

pub struct DistanceGrid {
    last: Option<GridData>,
    current: Option<GridData>,
    /// The cell `current` was measured from.
    measured_from: Option<HexPoint>,
}

impl DistanceGrid {
    pub fn new() -> Self {
        Self {
            last: None,
            current: None,
            measured_from: None,
        }
    }

    /// Distances are measured from the head's cell, so they only change when
    /// the head reaches a new one; in between, they fade from the previous map
    /// to the new one as the head crosses its cell (see [`Self::cells`]).
    pub fn update(&mut self, player_snake: &Snake, other_snakes: impl Snakes, board_dim: HexDim) {
        let head = player_snake.head().pos;
        if self.current.is_none() || self.measured_from != Some(head) {
            self.measured_from = Some(head);
            self.last = self
                .current
                .replace(find_distances(player_snake, other_snakes, board_dim));
        }
    }

    /// Every cell with a distance in the previous map or the current one, as
    /// `(cell, previous distance, current distance)`. Without a previous map
    /// the current one stands in for it, so nothing fades.
    pub fn cells(&self) -> impl Iterator<Item = (HexPoint, Option<Distance>, Option<Distance>)> + '_ {
        let current = self.current.as_ref();
        let last = self.last.as_ref().or(current);
        let before = last
            .into_iter()
            .flatten()
            .map(move |(pos, &dist)| (*pos, Some(dist), current.and_then(|current| current.get(pos).copied())));
        let new = current
            .into_iter()
            .flatten()
            .filter(move |(pos, _)| !last.is_some_and(|last| last.contains_key(pos)))
            .map(|(pos, &dist)| (*pos, None, Some(dist)));
        before.chain(new)
    }

    pub fn invalidate(&mut self) {
        self.last = None;
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(h: isize) -> HexPoint {
        HexPoint { h, v: 0 }
    }

    fn sorted_cells(grid: &DistanceGrid) -> Vec<(HexPoint, Option<Distance>, Option<Distance>)> {
        grid.cells().sorted_by_key(|(pos, _, _)| *pos).collect()
    }

    #[test]
    fn cells_in_either_map_fade_in_or_out() {
        let grid = DistanceGrid {
            last: Some(HashMap::from([(cell(0), 1.), (cell(1), 2.)])),
            current: Some(HashMap::from([(cell(1), 3.), (cell(2), 4.)])),
            measured_from: None,
        };
        assert_eq!(
            sorted_cells(&grid),
            [
                (cell(0), Some(1.), None),
                (cell(1), Some(2.), Some(3.)),
                (cell(2), None, Some(4.)),
            ]
        );
    }

    #[test]
    fn without_a_previous_map_nothing_fades() {
        let grid = DistanceGrid {
            last: None,
            current: Some(HashMap::from([(cell(0), 1.)])),
            measured_from: None,
        };
        assert_eq!(sorted_cells(&grid), [(cell(0), Some(1.), Some(1.))]);
    }
}
