use crate::app::hex::{Hex, Dir, HexPos, HexType};
use crate::app::snake::{SnakeState, Snake, SnakeController};
use crate::app::control::Controls;
use std::collections::VecDeque;
use ggez::event::KeyCode;
use crate::app::palette::SnakePalette;

pub enum PlayerSnakeType {
    SinglePlayer,
    Player1,
    Player2,
}

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
    fn next_dir(&mut self, dir: Dir) -> Dir {
        let new_dir = if let Some(queue_dir) = self.control_queue.pop_front() {
            queue_dir
        } else {
            dir
        };
        self.dir = new_dir;
        new_dir
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

pub struct PlayerSnake {
    snake_type: PlayerSnakeType,
    palette: SnakePalette,

    pub body: Vec<Hex>,
    growing: usize,
    dir: Dir,
    pub game_dim: HexPos,

    pub state: SnakeState,

    pub ctrl: Controls,
    ctrl_queue: VecDeque<Dir>,
}