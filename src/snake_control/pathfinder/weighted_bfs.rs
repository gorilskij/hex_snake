use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexPoint};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

#[derive(Clone)]
struct SearchPoint {
    parent: Option<Rc<Self>>,
    pos: HexPoint,
    dir: Dir,
    len: usize,
    /// cost = len + TURN_COST * num_turns + TELEPORT_COST * num_teleports
    cost: usize,
}

// reverse order iterator of positions
struct Iter<'a>(Option<&'a SearchPoint>);

impl Iterator for Iter<'_> {
    type Item = HexPoint;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0?.pos;
        self.0 = self.0?.parent.as_ref().map(|rc| rc.as_ref());
        Some(next)
    }
}

impl SearchPoint {
    const TURN_COST: usize = 5;
    const TELEPORT_COST: usize = 15;

    fn get_path(&self) -> Path {
        let mut vd = VecDeque::with_capacity(self.len);
        for pos in Iter(Some(self)) {
            vd.push_front(pos);
        }
        vd
    }
}

pub struct WeightedBFS;

impl PathFinder for WeightedBFS {
    fn get_path(
        &self,
        targets: &dyn Targets,
        body: &Body,
        knowledge: Option<&Knowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Path> {
        let off_limits: &HashSet<_> = &if let Some(pk) = knowledge {
            body.segments
                .iter()
                .chain(other_snakes.iter_segments())
                .filter(|seg| !pk.can_pass_through_self(seg))
                .map(|seg| seg.pos)
                .collect()
        } else {
            body.segments
                .iter()
                .chain(other_snakes.iter_segments())
                .map(|seg| seg.pos)
                .collect()
        };

        let shortest_paths: RefCell<HashMap<HexPoint, SearchPoint>> = Default::default();

        let mut generation = vec![SearchPoint {
            parent: None,
            pos: body.segments[0].pos,
            dir: body.dir,
            len: 1,
            cost: 1,
        }];

        let best: RefCell<Option<(HexPoint, usize)>> = Default::default();

        // bfs
        loop {
            // advance
            generation = generation
                .into_iter()
                .flat_map(|sp| {
                    let pos = sp.pos;
                    let rc = Rc::new(sp);

                    Dir::iter()
                        .scan(rc, move |rc, dir| {
                            let (new_pos, teleported) =
                                pos.explicit_wrapping_translate(dir, 1, gtx.board_dim);

                            if off_limits.contains(&new_pos) {
                                return None;
                            }

                            let new_sp = SearchPoint {
                                parent: Some(rc.clone()),
                                pos: new_pos,
                                dir,
                                len: rc.len + 1,
                                cost: rc.cost
                                    + if dir != rc.dir {
                                        SearchPoint::TURN_COST
                                    } else {
                                        0
                                    }
                                    + if teleported {
                                        SearchPoint::TELEPORT_COST
                                    } else {
                                        0
                                    },
                            };
                            Some(new_sp)
                        })
                        .filter(|sp| {
                            // keep track of the shortest path to each position
                            // match shortest_paths.borrow().get(&sp.pos) {
                            //     Some(other_sp) if other_sp.cost() <= sp.cost() => false,
                            //     other => {
                            //         drop(other);
                            //         shortest_paths.borrow_mut().insert(sp.pos, sp.clone());
                            //         match *best.borrow() {
                            //             Some((_, cost)) if cost <= sp.cost() => {}
                            //             _ => *best.borrow_mut() = Some((sp.pos, sp.cost()))
                            //         }
                            //         true
                            //     }
                            // }
                            let mut shortest_paths = shortest_paths.borrow_mut();
                            match shortest_paths.get_mut(&sp.pos) {
                                Some(other_sp) => {
                                    if sp.cost < other_sp.cost {
                                        *other_sp = sp.clone();
                                    } else {
                                        return false;
                                    }
                                }
                                None => {
                                    shortest_paths.insert(sp.pos, sp.clone());
                                }
                            };
                            if targets.iter().contains(&sp.pos) {
                                match &mut *best.borrow_mut() {
                                    Some((_, cost)) if *cost <= sp.cost => {}
                                    other => *other = Some((sp.pos, sp.cost)),
                                }
                            }
                            true
                        })
                })
                .collect();

            // check exit condition (when the live path with the lowest cost is successful)
            if let Some((best_pos, best_cost)) = *best.borrow() {
                let gen_min_cost = generation.iter().map(|sp| sp.cost).min();
                match gen_min_cost {
                    Some(cost) if cost < best_cost - 1 => {}
                    _ => return Some(shortest_paths.borrow()[&best_pos].get_path()),
                }
            }

            if generation.is_empty() {
                return None;
            }
        }
    }
}
