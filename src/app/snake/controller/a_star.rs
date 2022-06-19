use crate::{
    app::snake::{
        controller::{Controller, OtherSnakes},
        Body,
    },
    basic::{Dir, HexDim, HexPoint},
};

use crate::app::{apple::Apple, game_context::GameContext, snake::PassthroughKnowledge};
use ggez::Context;
use itertools::Itertools;
use rayon::prelude::*;
use std::{
    cmp::{max, min},
    collections::HashSet,
    rc::Rc,
};
use ggez::graphics::Mesh;
use ggez::winit::event::VirtualKeyCode;
use crate::app::app_error::AppResult;

pub struct AStar {
    pub passthrough_knowledge: PassthroughKnowledge,
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

    fn occupied_positions(&self, body: &Body, other_snakes: OtherSnakes) -> HashSet<HexPoint> {
        body
            .cells
            .par_iter()
            .filter(|seg| !self.passthrough_knowledge.can_pass_through_self(seg))
            .chain(other_snakes.par_iter_snakes().flat_map(|snake| {
                let checker = self.passthrough_knowledge.checker(&snake.snake_type);
                snake
                    .body
                    .cells
                    .par_iter()
                    .filter(move |seg| !checker.can_pass_through_other(seg))
            }))
            .map(|seg| seg.pos)
            .collect()
    }

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
        let head = body.cells[0].pos;

        let mut seen = HashSet::new();
        seen.insert(head);

        let mut paths = vec![PathNode {
            point: head,
            length: 0,
            parent: None,
        }];

        let forbidden_positions = {
            let mut fp = self.occupied_positions(&body, other_snakes);
            // pretend the position directly behind the head is occupied to avoid 180° turns
            fp.insert(body.cells[0].pos.translate(-body.dir, 1));
            fp
        };

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
            .cells
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
        gtx: &GameContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        if let Some(p) = self.target {
            if p == body.cells[0].pos {
                // apple eaten
                self.target = None;
            }
        }

        let going_to_crash = || {
            self
                .path
                .get(0)
                .map(|dir| {
                    let pos = body.cells[0].pos.translate(*dir, 1);
                    self.occupied_positions(&body, other_snakes).contains(&pos)
                })
                .unwrap_or(false)
        };

        if self.target.is_none()
            || self.path.is_empty()
            || self.steps_since_update >= Self::UPDATE_EVERY_N_STEPS
            || going_to_crash()
        {
            self.recalculate_target(body.cells[0].pos, apples, gtx.board_dim);
            self.recalculate_path(body, other_snakes, gtx.board_dim);
            self.steps_since_update = 0;
        }
        self.steps_since_update += 1;

        let head_pos = body.cells[0].pos;
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
            Self::least_damage(body.cells[0].pos, body, other_snakes, gtx.board_dim)
        }
    }

    fn passthrough_knowledge(&self) -> Option<&PassthroughKnowledge> {
        Some(&self.passthrough_knowledge)
    }
}
