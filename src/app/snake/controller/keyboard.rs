use crate::{
    app::{
        game::Apple,
        keyboard_control::Controls,
        snake::{
            controller::{Controller, OtherSnakes},
            SnakeBody,
        },
    },
    basic::{Dir, HexDim},
};
use ggez::event::KeyCode;
use std::collections::VecDeque;

pub struct Keyboard {
    pub controls: Controls,
    pub control_queue: VecDeque<Dir>,
    pub dir: Dir,
}

impl Keyboard {
    pub const CTRL_QUEUE_LIMIT: usize = 3;
}

impl Controller for Keyboard {
    fn next_dir(&mut self, _: &SnakeBody, _: OtherSnakes, _: &[Apple], _: HexDim) -> Option<Dir> {
        if let Some(queue_dir) = self.control_queue.pop_front() {
            self.dir = queue_dir;
            Some(self.dir)
        } else {
            None
        }
    }

    fn reset(&mut self, dir: Dir) {
        self.control_queue.clear();
        self.dir = dir;
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

        // deny 180deg turns
        if self.control_queue.is_empty() && self.dir != -new_dir
            || !self.control_queue.is_empty()
                && self.control_queue.len() < Self::CTRL_QUEUE_LIMIT
                && new_dir != -self.control_queue[self.control_queue.len() - 1]
        {
            self.control_queue.push_back(new_dir)
        }
    }
}
