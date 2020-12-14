use crate::app::hex::{Hex, Dir, HexPos, HexType};
use crate::app::snake::{SnakeState, Snake};
use crate::app::ctrl::Controls;
use std::collections::VecDeque;
use ggez::event::KeyCode;
use crate::app::palette::SnakePalette;

pub enum PlayerSnakeType {
    SinglePlayer,
    Player1,
    Player2,
}

pub struct PlayerSnake {
    snake_type: PlayerSnakeType,
    palette: SnakePalette,

    pub body: Vec<Hex>,
    growing: usize,
    dir: Dir,
    pub game_dim: HexPos, // cached value

    pub state: SnakeState,

    pub ctrl: Controls,
    ctrl_queue: VecDeque<Dir>,
}

impl Snake for PlayerSnake {
    fn body(&self) -> &Vec<Hex> {
        &self.body
    }

    fn palette(&self) -> &SnakePalette {
        &self.palette
    }

    fn advance(&mut self) {
        self.pop_ctrl_queue();

        let mut new_head = Hex {
            typ: HexType::Normal,
            pos: self.body[0].pos,
            teleported: None,
        };

        // todo make O(1)
        //  at the moment this just moves the head back until the last cell that's still in the map
        //  this could be done as a single calculation
        new_head.translate(self.dir, 1);
        let teleported;
        if !new_head.is_in(self.game_dim) {
            teleported = true;
            // find reappearance point
            new_head.translate(self.dir, -1);
            while new_head.is_in(self.game_dim) {
                new_head.translate(self.dir, -1);
            }
            new_head.translate(self.dir, 1);
        } else {
            teleported = false;
        }

        let body_last = self.body.len() - 1;
        if let HexType::Eaten(amount) = &mut self.body[body_last].typ {
            if *amount == 0 {
                self.body[body_last].typ = HexType::Normal;
            } else {
                self.growing += 1;
                *amount -= 1;
            }
        }

        if self.growing > 0 {
            self.body.insert(0, new_head);
            self.growing -= 1;
        } else {
            self.body.rotate_right(1);
            self.body[0] = new_head;
        }

        if teleported {
            self.body[0].teleported = Some(-self.dir);
            self.body[1].teleported = Some(self.dir);
        }
    }
}

impl PlayerSnake {
    const CTRL_QUEUE_LIMIT: usize = 3;

    pub fn new(snake_type: PlayerSnakeType, palette: SnakePalette, dim: HexPos, start_pos: HexPos, ctrl: Controls) -> Self {
        let head = Hex {
            typ: HexType::Normal,
            pos: start_pos,
            teleported: None,
        };

        Self {
            snake_type,
            palette,

            body: vec![head],
            growing: 15,
            dir: Dir::U,
            game_dim: dim,

            state: SnakeState::Living,

            ctrl,
            ctrl_queue: VecDeque::with_capacity(Self::CTRL_QUEUE_LIMIT),
        }
    }

    // -- control --
    pub fn key_pressed(&mut self, key: KeyCode) {
        use Dir::*;
        let new_dir = match key {
            k if k == self.ctrl.u => U,
            k if k == self.ctrl.d => D,
            k if k == self.ctrl.ul => UL,
            k if k == self.ctrl.ur => UR,
            k if k == self.ctrl.dl => DL,
            k if k == self.ctrl.dr => DR,
            _ => return,
        };

        if self.ctrl_queue.is_empty() && new_dir != -self.dir
            || !self.ctrl_queue.is_empty()
            && self.ctrl_queue.len() < Self::CTRL_QUEUE_LIMIT
            && new_dir != -self.ctrl_queue[self.ctrl_queue.len() - 1]
        {
            self.ctrl_queue.push_back(new_dir)
        }
    }

    fn pop_ctrl_queue(&mut self) {
        if let Some(new_dir) = self.ctrl_queue.pop_front() {
            self.dir = new_dir
        }
    }
}