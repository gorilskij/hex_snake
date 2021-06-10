use crate::{
    app::{
        game::Apple,
        snake::{
            controller::{Controller, OtherSnakes},
            SnakeBody,
        },
    },
    basic::{Dir, HexDim, HexPoint},
};

#[cfg(feature = "show_search_path")]
use crate::app::snake::controller::{ETHEREAL_PATH, ETHEREAL_SEEN};

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
    value: HexPoint,
    length: usize,
    parent: Option<Rc<Self>>,
}

impl AStar {
    const UPDATE_EVERY_N_STEPS: usize = 10;
    const HEURISTIC: fn(HexPoint, HexPoint, HexDim) -> usize = Self::heuristic3;
    const SEARCH_LIMIT: Option<usize> = None;

    #[allow(dead_code)]
    fn heuristic1(a: HexPoint, b: HexPoint, board_dim: HexDim) -> usize {
        let h_dist = *[
            (a.h - b.h).abs(),
            (a.h + board_dim.h - b.h).abs(),
            (b.h + board_dim.h - a.h).abs(),
        ]
        .iter()
        .min()
        .unwrap() as usize;
        let v_dist = *[
            (a.v - b.v).abs(),
            (a.v + board_dim.v - b.v).abs(),
            (b.v + board_dim.v - a.v).abs(),
        ]
        .iter()
        .min()
        .unwrap() as usize;
        max(h_dist, v_dist)
    }

    #[allow(dead_code)]
    fn heuristic2(a: HexPoint, b: HexPoint, board_dim: HexDim) -> usize {
        let h_dist = *[
            (a.h - b.h).abs(),
            (a.h + board_dim.h - b.h).abs(),
            (b.h + board_dim.h - a.h).abs(),
        ]
        .iter()
        .min()
        .unwrap() as usize;
        let v_dist = *[
            (a.v - b.v).abs(),
            (a.v + board_dim.v - b.v).abs(),
            (b.v + board_dim.v - a.v).abs(),
        ]
        .iter()
        .min()
        .unwrap() as usize;
        h_dist + v_dist
    }

    #[allow(dead_code)]
    fn heuristic3(a: HexPoint, b: HexPoint, board_dim: HexDim) -> usize {
        let h_dist = *[
            (a.h - b.h).abs(),
            (a.h + board_dim.h - b.h).abs(),
            (b.h + board_dim.h - a.h).abs(),
        ]
        .iter()
        .min()
        .unwrap();
        let v_dist = *[
            (a.v - b.v).abs(),
            (a.v + board_dim.v - b.v).abs(),
            (b.v + board_dim.v - a.v).abs(),
        ]
        .iter()
        .min()
        .unwrap();
        let h3 = max(max(h_dist, v_dist - h_dist / 2), 0) as usize;
        min(HexPoint::manhattan_distance(a, b), h3)
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

    fn recalculate_path(
        &mut self,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        board_dim: HexDim,
    ) {
        let target = match self.target {
            Some(p) => p,
            None => {
                self.path.clear();
                return;
            }
        };

        // A* search

        let head = snake_body[0].pos;

        let mut seen = HashSet::new();
        seen.insert(head);
        // the last node in each path is the newest
        // let mut paths = vec![vec![head]];
        let mut paths = vec![PathNode {
            value: head,
            length: 0,
            parent: None,
        }];

        let forbidden_positions = snake_body
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect::<HashSet<_>>();

        loop {
            // select which node to expand
            // f(x) = length of path
            // g(x) = non-wrapping manhattan distance to target
            let expand_idx = paths
                .iter()
                .filter(|path| {
                    Self::SEARCH_LIMIT
                        .map(|limit| path.length < limit)
                        .unwrap_or(true)
                })
                .position_min_by_key(|path| {
                    path.length + Self::HEURISTIC(path.value, target, board_dim)
                });
            let expand_idx = match expand_idx {
                Some(idx) => idx,
                None => {
                    match paths
                        .iter()
                        .min_by_key(|path| Self::HEURISTIC(path.value, target, board_dim))
                    {
                        Some(best_path) => {
                            // get path (reversed)
                            let mut vec_path = Vec::with_capacity(best_path.length + 1);
                            let mut current = Some(best_path);
                            // include snake head
                            while let Some(c) = current {
                                vec_path.push(c.value);
                                current = c.parent.as_ref().map(|rc| Rc::as_ref(rc));
                            }

                            // calculate directions
                            self.path = vec_path
                                .iter()
                                .rev()
                                .zip(vec_path.iter().rev().skip(1))
                                .map(|(a, b)| {
                                    a.wrapping_dir_to_1(*b, board_dim)
                                        .unwrap_or_else(|| panic!("no dir from {:?} to {:?}", a, b))
                                })
                                .collect();

                            #[cfg(feature = "show_search_path")]
                            unsafe {
                                vec_path.reverse();
                                ETHEREAL_PATH = Some(vec_path);
                                ETHEREAL_SEEN = Some(seen);
                            }

                            return;
                        }
                        None => self.path.clear(),
                    }

                    return;
                }
            };

            let path = paths.remove(expand_idx);

            if path.value == target {
                // get path (reversed)
                let mut vec_path = Vec::with_capacity(path.length + 1);
                let mut current = Some(&path);
                // include snake head
                while let Some(c) = current {
                    vec_path.push(c.value);
                    current = c.parent.as_ref().map(|rc| Rc::as_ref(rc));
                }

                // calculate directions
                self.path = vec_path
                    .iter()
                    .rev()
                    .zip(vec_path.iter().rev().skip(1))
                    .map(|(a, b)| {
                        a.wrapping_dir_to_1(*b, board_dim)
                            .unwrap_or_else(|| panic!("no dir from {:?} to {:?}", a, b))
                    })
                    .collect();

                #[cfg(feature = "show_search_path")]
                unsafe {
                    vec_path.reverse();
                    ETHEREAL_PATH = Some(vec_path);
                    ETHEREAL_SEEN = Some(seen);
                }

                return;
            }

            let new_length = path.length + 1;
            let parent = Rc::new(path);
            for dir in Dir::iter() {
                let candidate = parent.value.wrapping_translate(dir, 1, board_dim);
                if forbidden_positions.contains(&candidate) || seen.contains(&candidate) {
                    continue;
                }
                seen.insert(candidate);

                let new_path = PathNode {
                    value: candidate,
                    length: new_length,
                    parent: Some(Rc::clone(&parent)),
                };
                paths.push(new_path);
                // inefficient path cloning
                // let mut new_path = path.clone();
                // new_path.push(candidate);
                // paths.push(new_path);
            }
        }
    }

    // if there's no best path, at least avoid running into something
    fn least_damage(
        head: HexPoint,
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        board_dim: HexDim,
    ) -> Option<Dir> {
        let forbidden = snake_body
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect::<HashSet<_>>();
        let next = head.wrapping_translate(snake_body.dir, 1, board_dim);
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
        snake_body: &SnakeBody,
        other_snakes: OtherSnakes,
        apples: &[Apple],
        board_dim: HexDim,
    ) -> Option<Dir> {
        if let Some(p) = self.target {
            if p == snake_body[0].pos {
                // apple eaten
                self.target = None;
            }
        }

        if self.target.is_none()
            || self.path.is_empty()
            || self.steps_since_update >= Self::UPDATE_EVERY_N_STEPS
        {
            self.recalculate_target(snake_body[0].pos, apples, board_dim);
            self.recalculate_path(snake_body, other_snakes, board_dim);
            self.steps_since_update = 0;
        }
        self.steps_since_update += 1;

        #[cfg(feature = "show_search_path")]
        unsafe {
            match &mut ETHEREAL_PATH {
                Some(v) if !v.is_empty() => drop(v.remove(0)),
                _ => ETHEREAL_PATH = None,
            }
            match &mut ETHEREAL_SEEN {
                Some(h) => drop(h.remove(&snake_body[0].pos)),
                _ => {}
            }
        }

        if !self.path.is_empty() {
            let dir = self.path.remove(0);
            Some(dir)
        } else {
            Self::least_damage(snake_body[0].pos, snake_body, other_snakes, board_dim)
        }
    }
}

#[cfg(feature = "show_search_path")]
impl Drop for AStar {
    fn drop(&mut self) {
        unsafe {
            ETHEREAL_PATH = None;
            ETHEREAL_SEEN = None;
        }
    }
}
