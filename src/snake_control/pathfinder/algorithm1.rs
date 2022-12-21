use super::{Path, PathFinder};
use crate::app::game_context::GameContext;
use crate::basic::{Dir, HexPoint};
use crate::snake::{Body, PassthroughKnowledge};
use crate::view::snakes::Snakes;
use crate::view::targets::Targets;
use itertools::Itertools;
use map_with_state::IntoMapWithState;
use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

// TODO: instead of cloning, use more Rcs
#[derive(Clone)]
struct SearchPoint {
    parent: Option<Rc<Self>>,
    pos: HexPoint,
    dir: Dir,
    len: usize,
    num_turns: usize,
    num_teleports: usize,
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

    fn cost(&self) -> usize {
        self.len + self.num_turns * Self::TURN_COST + self.num_teleports * Self::TELEPORT_COST
    }

    fn get_path(&self) -> Path {
        let mut vd = VecDeque::with_capacity(self.len);
        for pos in Iter(Some(self)) {
            vd.push_front(pos);
        }
        vd
    }
}

// TODO: rename
pub struct Algorithm1;

impl PathFinder for Algorithm1 {
    fn get_path(
        &self,
        targets: &dyn Targets,
        body: &Body,
        passthrough_knowledge: Option<&PassthroughKnowledge>,
        other_snakes: &dyn Snakes,
        gtx: &GameContext,
    ) -> Option<Path> {
        let off_limits: HashSet<_> = if let Some(pk) = passthrough_knowledge {
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
            num_turns: 0,
            num_teleports: 0,
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
                    let rc_clone = rc.clone();

                    Dir::iter()
                        .map(move |dir| {
                            let (new_pos, teleported) =
                                pos.explicit_wrapping_translate(dir, 1, gtx.board_dim);
                            let turned = dir != rc.dir;
                            (dir, new_pos, turned, teleported)
                        })
                        .filter(|(_, new_pos, _, _)| !off_limits.contains(new_pos))
                        .map_with_state(rc_clone, |rc, (dir, new_pos, turned, teleported)| {
                            let new_sp = SearchPoint {
                                parent: Some(rc.clone()),
                                pos: new_pos,
                                dir,
                                len: rc.len + 1,
                                num_turns: rc.num_turns + turned as usize,
                                num_teleports: rc.num_teleports + teleported as usize,
                            };
                            (rc, new_sp)
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
                            match shortest_paths.borrow_mut().entry(sp.pos) {
                                Occupied(mut occupied) => {
                                    let other_sp = occupied.get();
                                    if sp.cost() < other_sp.cost() {
                                        occupied.insert(sp.clone());
                                        if targets.iter().contains(&sp.pos) {
                                            match &mut *best.borrow_mut() {
                                                Some((_, cost)) if *cost <= sp.cost() => {}
                                                other => *other = Some((sp.pos, sp.cost())),
                                            }
                                        }
                                        true
                                    } else {
                                        false
                                    }
                                }
                                Vacant(vacant) => {
                                    vacant.insert(sp.clone());
                                    if targets.iter().contains(&sp.pos) {
                                        match &mut *best.borrow_mut() {
                                            Some((_, cost)) if *cost <= sp.cost() => {}
                                            other => *other = Some((sp.pos, sp.cost())),
                                        }
                                    }
                                    true
                                }
                            }
                        })
                })
                .collect();

            // check exit condition (when the live path with the lowest cost is successful)
            if let Some((best_pos, best_cost)) = *best.borrow() {
                let gen_min_cost = generation.iter().map(|sp| sp.cost()).min();
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
