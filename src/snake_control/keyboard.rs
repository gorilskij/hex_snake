use std::collections::VecDeque;

use crate::app::game_context::GameContext;
use crate::app::key::Key;
use crate::apple::Apple;
use crate::basic::{Dir, Side};
use crate::snake::eat_mechanics::Knowledge;
use crate::snake::Body;
use crate::snake_control::Controller;
use crate::view::snakes::Snakes;

pub struct Keyboard {
    /// Whose keys (see [`crate::app::prefs::Prefs::controls`]) this snake
    /// answers to; `None` for the single player, whose side is a preference
    side: Option<Side>,
    control_queue: VecDeque<Dir>,
    dir: Dir,
    // whether change of direction was deferred from the previous cell,
    // this forces it to happen on the next cell no matter what, this
    // prevents infinite deferral for when frame_frac is always high
    // (high speed and laggy situations)
    deferred: bool,

    // assumes the player knows everything
    knowledge: Knowledge,
}

impl Keyboard {
    // How many moves ahead a player can make (this allows quick 180° turns)
    const CTRL_QUEUE_LIMIT: usize = 3;
    /// If frame_fraction is greater than this value, the change of
    /// direction is deferred to the next cell, this prevents abrupt
    /// jumps of the snake head
    const LAST_ACTIONABLE_THRESHOLD: f32 = 0.85;

    pub fn new(side: Option<Side>, start_dir: Dir, knowledge: Knowledge) -> Self {
        Self {
            side,
            control_queue: VecDeque::with_capacity(Self::CTRL_QUEUE_LIMIT),
            dir: start_dir,
            deferred: false,

            knowledge,
        }
    }
}

impl Controller for Keyboard {
    fn next_dir(
        &mut self,
        body: &mut Body,
        _: Option<&Knowledge>,
        _: &dyn Snakes,
        _: &[Apple],
        _: &GameContext,
    ) -> Option<Dir> {
        if self.deferred || body.head_fraction < Self::LAST_ACTIONABLE_THRESHOLD {
            self.deferred = false;
            if let Some(dir) = self.control_queue.pop_front() {
                self.dir = dir;
                return Some(self.dir);
            }
        } else {
            self.deferred = true;
            // don't ask until the next frame
            return Some(body.dir);
        }
        None
    }

    fn reset(&mut self, dir: Dir) {
        self.control_queue.clear();
        self.dir = dir;
    }

    fn key_pressed(&mut self, key: Key, gtx: &GameContext) {
        // looked up on every press, so rebinding applies at once
        let side = self.side.unwrap_or(gtx.prefs.single_player);
        let Some(new_dir) = gtx.prefs.controls(side).dir_of(key) else {
            return;
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

    fn knowledge(&self) -> Option<&Knowledge> {
        Some(&self.knowledge)
    }
}
