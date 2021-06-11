use crate::{
    app::{
        game::Apple,
        snake::{
            controller::{Controller, OtherSnakes},
            SnakeBody,
        },
    },
    basic::{Dir, HexPoint},
};
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
    fn next_dir(&mut self, _: &SnakeBody, _: OtherSnakes, _: &[Apple], _: HexPoint) -> Option<Dir> {
        if self.wait > 0 {
            self.wait -= 1;
        } else {
            match self.move_sequence[self.next_move_idx] {
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