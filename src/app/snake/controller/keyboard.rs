use crate::{app::{
    apple::Apple,
    game_context::GameContext,
    keyboard_control::Controls,
    snake::{
        controller::{Controller, OtherSnakes},
        Body,
    },
}, basic::Dir, ControlSetup};
use ggez::{event::KeyCode, Context};
use std::collections::VecDeque;

pub struct Keyboard {
    controls: Controls,
    control_queue: VecDeque<Dir>,
    dir: Dir,
    // whether change of direction was deferred from the previous cell,
    // this forces it to happen on the next cell no matter what, this
    // prevents infinite deferral for when frame_frac is always high
    // (high speed and laggy situations)
    deferred: bool,
}

impl Keyboard {
    // How many moves ahead a player can make (this allows quick 180° turns)
    const CTRL_QUEUE_LIMIT: usize = 3;
    // After frame_fraction is greater than this value, the change of
    // direction is deferred to the next cell, this gives smoother motion
    const LAST_ACTIONABLE_THRESHOLD: f32 = 0.85;

    pub fn new(control_setup: ControlSetup, start_dir: Dir) -> Self {
        Self {
            controls: control_setup.into(),
            control_queue: VecDeque::with_capacity(Self::CTRL_QUEUE_LIMIT),
            dir: start_dir,
            deferred: false,
        }
    }
}

impl Controller for Keyboard {
    fn next_dir(
        &mut self,
        _: &mut Body,
        _: OtherSnakes,
        _: &[Apple],
        gtx: &GameContext,
        _: &Context,
    ) -> Option<Dir> {
        if self.deferred || gtx.frame_stamp.1 < Self::LAST_ACTIONABLE_THRESHOLD {
            self.deferred = false;
            if let Some(queue_dir) = self.control_queue.pop_front() {
                self.dir = queue_dir;
                return Some(self.dir)
            }
        } else {
            self.deferred = true;
        }
        None
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
