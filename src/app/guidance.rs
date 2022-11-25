use crate::{
    app::game_context::GameContext,
    apple::Apple,
    basic::{Dir, HexDim, HexPoint},
    snake::Snake,
    view::snakes::Snakes,
};
use ggez::Context;
use itertools::Itertools;
use map_with_state::IntoMapWithState;
use std::{
    cell::RefCell,
    cmp::min,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap, HashSet,
    },
    rc::Rc,
};

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
    fn get_path(&self) -> Vec<HexPoint> {
        let mut vec = vec![HexPoint { h: 0, v: 0 }; self.len];
        for (i, pos) in Iter(Some(self)).enumerate() {
            vec[self.len - i - 1] = pos;
        }
        vec
    }
}

impl SearchPoint {
    const TURN_COST: usize = 5;
    const TELEPORT_COST: usize = 15;

    fn cost(&self) -> usize {
        self.len + self.num_turns * Self::TURN_COST + self.num_teleports * Self::TELEPORT_COST
    }

    fn is_successful(&self, apples: &[Apple]) -> bool {
        apples.iter().any(|apple| apple.pos == self.pos)
    }
}

pub fn get_guidance_path(
    player_snake: &Snake,
    other_snakes: impl Snakes,
    apples: &[Apple],
    ctx: &mut Context,
    gtx: &GameContext,
) -> Vec<HexPoint> {
    let off_limits: HashSet<_> =
        if let Some(passthrough_knowledge) = player_snake.controller.passthrough_knowledge() {
            player_snake
                .body
                .segments
                .iter()
                .chain(other_snakes.iter_segments())
                .filter(|seg| !passthrough_knowledge.can_pass_through_self(seg))
                .map(|seg| seg.pos)
                .collect()
        } else {
            player_snake
                .body
                .segments
                .iter()
                .chain(other_snakes.iter_segments())
                .map(|seg| seg.pos)
                .collect()
        };

    let shortest_paths: RefCell<HashMap<HexPoint, SearchPoint>> = Default::default();

    let mut generation = vec![SearchPoint {
        parent: None,
        pos: player_snake.head().pos,
        dir: player_snake.body.dir,
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
                                    if sp.is_successful(apples) {
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
                            Vacant(mut vacant) => {
                                vacant.insert(sp.clone());
                                if sp.is_successful(apples) {
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
                _ => return shortest_paths.borrow()[&best_pos].get_path(),
            }
        }

        // failsafe
        assert!(!generation.is_empty(), "exhaustive search found no apples");
    }
}
