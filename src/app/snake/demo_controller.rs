use crate::app::hex::Dir;
use crate::app::snake::SnakeController;

pub enum SimMove {
    Move(Dir),
    Wait(usize),
}

pub struct DemoController {
    move_sequence: Vec<SimMove>,
    next_move_idx: usize,
    wait: usize,
}

impl DemoController {
    pub fn new(move_sequence: Vec<SimMove>) -> Self {
        Self {
            move_sequence,
            next_move_idx: 0,
            wait: 0,
        }
    }
}

impl SnakeController for DemoController {
    fn next_dir(&mut self, dir: Dir) -> Dir {
        if self.wait > 0 {
            self.wait -= 1;
            dir
        } else {
            let new_dir = match self.move_sequence[self.next_move_idx] {
                SimMove::Wait(wait) => {
                    self.wait = wait;
                    dir
                },
                SimMove::Move(new_dir) => new_dir,
            };
            self.next_move_idx += 1;
            self.next_move_idx %= self.move_sequence.len();
            new_dir
        }
    }
}