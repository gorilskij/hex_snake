use crate::{
    app::snake::{
        controller::{Controller, OtherSnakes},
        Body,
    },
    basic::{Dir, HexDim, HexPoint},
};

use crate::app::apple::Apple;
use itertools::Itertools;
use std::{
    cmp::{max, min},
    collections::HashSet,
    rc::Rc,
};

pub struct AStar {
    pub target: Option<HexPoint>,
    pub path: Vec<Dir>,
    pub steps_since_update: usize,
}

struct PathNode {
    /// Where the path reaches
    point: HexPoint,
    length: usize,
    parent: Option<Rc<Self>>,
}

impl PathNode {
    // Directions to take to follow this path
    fn to_dir_vec(&self, board_dim: HexDim) -> Vec<Dir> {
        let mut vec = vec![Dir::U; self.length];
        // fill vec in reverse order
        let mut former = Some(self);
        for i in (0..self.length).rev() {
            let latter = former;
            former = former.and_then(|n| n.parent.as_ref().map(Rc::as_ref));
            vec[i] = former
                .unwrap()
                .point
                .wrapping_dir_to_1(latter.unwrap().point, board_dim)
                .unwrap()
        }

        assert_eq!(vec.len(), self.length);
        vec
    }
}

impl AStar {
    const UPDATE_EVERY_N_STEPS: usize = 10;
    const HEURISTIC: fn(HexPoint, HexPoint, HexDim) -> usize = Self::heuristic;
    // const SEARCH_LIMIT: Option<usize> = None;

    fn heuristic(a: HexPoint, b: HexPoint, board_dim: HexDim) -> usize {
        let h1 = (a.h - b.h).abs();
        let h2 = (a.h + board_dim.h - b.h).abs();
        let h3 = (b.h + board_dim.h - a.h).abs();
        let h_dist = min(h1, min(h2, h3));

        let v1 = (a.v - b.v).abs();
        let v2 = (a.v + board_dim.v - b.v).abs();
        let v3 = (b.v + board_dim.v - a.v).abs();
        let v_dist = min(v1, min(v2, v3));

        max(h_dist, v_dist - h_dist / 2) as usize
    }

    fn recalculate_target(&mut self, head: HexPoint, apples: &[Apple], board_dim: HexDim) {
        let new_target = apples
            .iter()
            .map(|a| a.pos)
            .min_by_key(|p| Self::HEURISTIC(head, *p, board_dim));
        if new_target != self.target {
            self.target = new_target;
            self.path.clear();
        }
    }

    fn recalculate_path(&mut self, body: &Body, other_snakes: OtherSnakes, board_dim: HexDim) {
        let target = match self.target {
            Some(p) => p,
            None => {
                self.path.clear();
                return;
            }
        };

        // A* search
        let head = body[0].pos;

        let mut seen = HashSet::new();
        seen.insert(head);

        let mut paths = vec![PathNode {
            point: head,
            length: 0,
            parent: None,
        }];

        let mut forbidden_positions: HashSet<_> = body
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect();
        // no 180° turns
        forbidden_positions.insert(body[0].pos.translate(-body.dir, 1));
        let forbidden_positions = forbidden_positions;

        loop {
            if paths.is_empty() {
                self.path = vec![];
                return;
            }

            let expand_idx = paths
                .iter()
                .position_min_by_key(|path| {
                    path.length + Self::HEURISTIC(path.point, target, board_dim)
                })
                .unwrap();

            let path = paths.remove(expand_idx);

            if path.point == target {
                self.path = path.to_dir_vec(board_dim);
                return;
            }

            let new_length = path.length + 1;
            let parent = Rc::new(path);
            for dir in Dir::iter() {
                let candidate = parent.point.wrapping_translate(dir, 1, board_dim);
                if forbidden_positions.contains(&candidate) || seen.contains(&candidate) {
                    continue;
                }
                seen.insert(candidate);

                let new_path = PathNode {
                    point: candidate,
                    length: new_length,
                    parent: Some(Rc::clone(&parent)),
                };
                paths.push(new_path);
            }
        }
    }

    // if there's no best path, at least avoid running into something
    fn least_damage(
        head: HexPoint,
        body: &Body,
        other_snakes: OtherSnakes,
        board_dim: HexDim,
    ) -> Option<Dir> {
        let forbidden = body
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect::<HashSet<_>>();
        let next = head.wrapping_translate(body.dir, 1, board_dim);
        if forbidden.contains(&next) {
            Dir::iter()
                .find(|dir| !forbidden.contains(&head.wrapping_translate(*dir, 1, board_dim)))
        } else {
            None
        }
    }
}

impl Controller for AStar {
    fn next_dir(
        &mut self,
        body: &mut Body,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir> {
        if let Some(p) = self.target {
            if p == body[0].pos {
                // apple eaten
                self.target = None;
            }
        }

        let going_to_crash = || {
            let potential_next_head = self.path.get(0).map(|dir| body[0].pos.translate(*dir, 1));
            potential_next_head
                .map(|pos| {
                    body.cells
                        .iter()
                        .chain(other_snakes.iter_segments())
                        .map(|seg| seg.pos)
                        .contains(&pos)
                })
                .unwrap_or(false)
        };

        if self.target.is_none()
            || self.path.is_empty()
            || self.steps_since_update >= Self::UPDATE_EVERY_N_STEPS
            || going_to_crash()
        {
            self.recalculate_target(body[0].pos, apples, board_dim);
            self.recalculate_path(body, other_snakes, board_dim);
            self.steps_since_update = 0;
        }
        self.steps_since_update += 1;

        let head_pos = body[0].pos;
        if let Some(search_trace) = &mut body.search_trace {
            search_trace.cells_searched.remove(&head_pos);
            if !search_trace.current_path.is_empty() {
                search_trace.current_path.remove(0);
            }
        }

        if !self.path.is_empty() {
            let dir = self.path.remove(0);
            Some(dir)
        } else {
            Self::least_damage(body[0].pos, body, other_snakes, board_dim)
        }
    }
}
