use crate::{
    app::{
        apple::Apple,
        keyboard_control::Controls,
        snake::{
            controller::{Controller, OtherSnakes},
            Body,
        },
    },
    basic::Dir,
};
use ggez::event::KeyCode;
use std::collections::VecDeque;
use ggez::Context;
use crate::app::game_context::GameContext;

pub struct Keyboard {
    pub controls: Controls,
    pub control_queue: VecDeque<Dir>,
    pub dir: Dir,
}

impl Keyboard {
    pub const CTRL_QUEUE_LIMIT: usize = 3;
}

impl Controller for Keyboard {
    fn next_dir(&mut self, _: &mut Body, _: OtherSnakes, _: &[Apple], _: &GameContext, _: &Context) -> Option<Dir> {
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
            k if k == self.controls.ul => Ul,
            k if k == self.controls.ur => Ur,
            k if k == self.controls.dl => Dl,
            k if k == self.controls.dr => Dr,
            _ => return,
        };

        // deny 180deg and 360deg turns
        if self.control_queue.is_empty() {
            if new_dir != self.dir && new_dir != -self.dir {
                self.control_queue.push_back(new_dir);
            }
        } else if self.control_queue.len() < Self::CTRL_QUEUE_LIMIT {
            let last_dir = self.control_queue[self.control_queue.len() - 1];
            if new_dir != last_dir && new_dir != -last_dir {
                self.control_queue.push_back(new_dir);
            }
        }
    }
}
