use std::collections::HashSet;
use std::rc::Rc;
use ggez::Context;
use itertools::Itertools;
use crate::app::game_context::GameContext;
use crate::app::snake::Snake;
use crate::app::snake::utils::OtherSnakes;
use crate::basic::{Dir, HexDim, HexPoint};
use map_with_state::IntoMapWithState;
use crate::app::apple::Apple;


struct SearchPoint {
    parent: Option<Rc<Self>>,
    pos: HexPoint,
    dir: Dir,
    turns: usize,
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
    // blind trust with the length
    fn get_path(&self, length: usize) -> Vec<HexPoint> {
        let mut  vec = vec![HexPoint { h: 0, v: 0 }; length];
        for (i, pos) in Iter(Some(self)).enumerate() {
            vec[length - i - 1] = pos;
        }
        vec
    }
}

pub fn get_guidance_path(
    player_snake: &Snake,
    other_snakes: OtherSnakes,
    apples: &[Apple],
    ctx: &mut Context,
    gtx: &GameContext,
) -> Vec<HexPoint> {
    let mut seen: HashSet<_> = if let Some(passthrough_knowledge) = player_snake.controller.passthrough_knowledge() {
        player_snake.body.cells
            .iter()
            .chain(other_snakes.iter_segments())
            .filter(|seg| !passthrough_knowledge.can_pass_through_self(seg))
            .map(|seg| seg.pos)
            .collect()
    } else {
        player_snake.body.cells
            .iter()
            .chain(other_snakes.iter_segments())
            .map(|seg| seg.pos)
            .collect()
    };
    let mut generation = vec![SearchPoint {
        parent: None,
        pos: player_snake.head().pos,
        dir: player_snake.body.dir,
        turns: 0,
    }];
    let mut length: usize = 1;

    // bfs
    loop {
        // advance
        generation = generation.into_iter()
            .flat_map(|sp| {
                let pos = sp.pos;
                let rc = Rc::new(sp);
                Dir::iter()
                    .map(move |dir| (dir, pos.wrapping_translate(dir, 1, gtx.board_dim)))
                    .filter(|(_, new_pos)| !seen.contains(new_pos))
                    .map_with_state(rc, |rc, (dir, new_pos)| {
                        let new_sp = SearchPoint {
                            parent: Some(rc.clone()),
                            pos: new_pos,
                            dir,
                            turns: if dir == rc.dir { rc.turns } else { rc.turns + 1 },
                        };
                        (rc, new_sp)
                    })
            })
            .group_by(|sp| sp.pos)
            .into_iter()
            .map(|(_, group)| {
                group.min_by_key(|sp| sp.turns).unwrap()
            })
            .collect();
        length += 1;

        seen.extend(generation.iter().map(|sp| sp.pos));

        // check exit condition
        for sp in &generation {
            for apple in apples {
                if sp.pos == apple.pos {
                    return sp.get_path(length);
                }
            }
        }

        // failsafe
        assert!(!generation.is_empty(), "exhaustive search found no apples");
    }
}
