use ggez::Context;

use crate::app::fps_control::FpsContext;
use crate::app::game_context::GameContext;
use crate::apple::Apple;
use crate::basic::Dir;
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;
#[allow(unused_macros)]
macro_rules! move_sequence {
    (@ turn($dir:expr) ) => {
        crate::app::snake::controller::SimMove::Turn($dir)
    };
    (@ wait($t:expr) ) => {
        crate::app::snake::controller::SimMove::Wait($t)
    };
    [ $( $action:tt ( $( $inner:tt )* ) ),* $(,)? ] => {
        vec![$(
            move_sequence!(@ $action( $( $inner )* ))
        ),*]
    };
}

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Turn(Dir),
    Wait(usize),
}

pub struct Programmed {
    pub move_sequence: Vec<Move>,
    pub dir: Dir,
    pub next_move_idx: usize,
    pub wait: usize,
}

impl Controller for Programmed {
    fn next_dir(
        &mut self,
        _: &mut Body,
        _: Option<&Knowledge>,
        _: &dyn Snakes,
        _: &[Apple],
        _: &GameContext,
        _: &FpsContext,
        _: &Context,
    ) -> Option<Dir> {
        if self.wait > 0 {
            self.wait -= 1;
        } else {
            match *self.move_sequence.get(self.next_move_idx)? {
                Move::Wait(wait) => self.wait = wait - 1,
                Move::Turn(new_dir) => self.dir = new_dir,
            };

            self.next_move_idx += 1;
            self.next_move_idx %= self.move_sequence.len();
        }

        Some(self.dir)
    }

    fn reset(&mut self, dir: Dir) {
        self.dir = dir;
        self.next_move_idx = 0;
        self.wait = 0;
    }
}
