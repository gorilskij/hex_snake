use std::collections::VecDeque;

use ggez::event::KeyCode;

use crate::app::control::Controls;
use crate::app::hex::{Dir, HexPos};
use crate::app::snake::{SnakeController, Snake, SnakeRepr};

pub struct PlayerController {
    controls: Controls,
    control_queue: VecDeque<Dir>,
    dir: Dir,
}

impl PlayerController {
    const CTRL_QUEUE_LIMIT: usize = 3;

    pub fn new(controls: Controls) -> Self {
        Self {
            controls,
            control_queue: VecDeque::with_capacity(Self::CTRL_QUEUE_LIMIT),
            dir: Dir::U,
        }
    }
}

impl SnakeController for PlayerController {
    fn next_dir(&mut self, snake: &SnakeRepr, _other_snakes: Vec<&SnakeRepr>, _apples: &[HexPos], board_dim: HexPos) -> Option<Dir> {
        if let Some(queue_dir) = self.control_queue.pop_front() {
            self.dir = queue_dir;
            Some(queue_dir)
        } else {
            None
        }
    }

    fn key_pressed(&mut self, key: KeyCode) {
        use Dir::*;
        let new_dir = match key {
            k if k == self.controls.u => U,
            k if k == self.controls.d => D,
            k if k == self.controls.ul => UL,
            k if k == self.controls.ur => UR,
            k if k == self.controls.dl => DL,
            k if k == self.controls.dr => DR,
            _ => return,
        };

        if self.control_queue.is_empty() && new_dir != -self.dir
            || !self.control_queue.is_empty()
            && self.control_queue.len() < Self::CTRL_QUEUE_LIMIT
            && new_dir != -self.control_queue[self.control_queue.len() - 1]
        {
            self.control_queue.push_back(new_dir)
        }
    }
}