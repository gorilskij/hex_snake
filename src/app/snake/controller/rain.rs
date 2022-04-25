use crate::{
    app::{
        apple::Apple,
        game_context::GameContext,
        snake::{
            controller::{rain::find3::Find3, Controller, OtherSnakes},
            Body, Type,
        },
    },
    basic::Dir,
};
use ggez::Context;

pub struct Rain;

// custom lazy iterator
mod find3 {
    pub struct Find3<I: Iterator> {
        iter: I,
        find_d: I::Item,
        found_d: bool,
        find_dl: I::Item,
        found_dl: bool,
        find_dr: I::Item,
        found_dr: bool,
    }

    impl<I: Iterator> Find3<I>
    where
        I::Item: Eq + Copy,
    {
        pub fn new(iter: I, find_d: I::Item, find_dl: I::Item, find_dr: I::Item) -> Self {
            Self {
                iter,
                find_d,
                found_d: false,
                find_dl,
                found_dl: false,
                find_dr,
                found_dr: false,
            }
        }

        fn consume_until(&mut self, find: I::Item) {
            for x in &mut self.iter {
                if !self.found_d && x == self.find_d {
                    self.found_d = true;
                }
                if !self.found_dl && x == self.find_dl {
                    self.found_dl = true;
                }
                if !self.found_dr && x == self.find_dr {
                    self.found_dr = true;
                }
                if x == find {
                    return;
                }
            }
        }

        pub fn contains_d(&mut self) -> bool {
            self.consume_until(self.find_d);
            self.found_d
        }

        pub fn contains_dl(&mut self) -> bool {
            self.consume_until(self.find_dl);
            self.found_dl
        }

        pub fn contains_dr(&mut self) -> bool {
            self.consume_until(self.find_dr);
            self.found_dr
        }
    }
}

impl Controller for Rain {
    fn next_dir(
        &mut self,
        body: &mut Body,
        other_snakes: OtherSnakes,
        _: &[Apple],
        gtx: &GameContext,
        _ctx: &Context,
    ) -> Option<Dir> {
        if body.cells[0].pos.v == gtx.board_dim.v - 1 {
            // todo!("return die")
            eprintln!(
                "TODO: suicide (or even better, disappear) -- {}",
                gtx.board_dim.v - 1
            );
            return None;
        }

        // TODO: randomize, make dl and dr equal probability
        // if possible, go down, else try to go down left or down right, else crash
        let next_d = body.cells[0].pos.translate(Dir::D, 1);
        let next_dl = body.cells[0]
            .pos
            .wrapping_translate(Dir::Dl, 1, gtx.board_dim);
        let next_dr = body.cells[0]
            .pos
            .wrapping_translate(Dir::Dr, 1, gtx.board_dim);

        let it = other_snakes
            .iter_snakes()
            .filter(|s| s.snake_type == Type::Rain)
            .flat_map(|s| s.body.cells.iter().map(|c| c.pos));

        let mut find3 = Find3::new(it, next_d, next_dl, next_dr);

        if !find3.contains_d() {
            Some(Dir::D)
        } else if !find3.contains_dl() {
            Some(Dir::Dl)
        } else if !find3.contains_dr() {
            Some(Dir::Dr)
        } else {
            None
        }
    }
}
