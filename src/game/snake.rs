use super::hex::{Hex, HexPos, HexType::*};
use crate::game::{ctrl::Controls, hex::HexType, theme::Palette};
use ggez::{event::KeyCode, graphics::Color, GameResult};
use std::{collections::VecDeque, ops::Neg};
use Dir::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Dir {
    U,
    D,
    UL,
    UR,
    DL,
    DR,
}

impl Neg for Dir {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            U => D,
            D => U,
            UL => DR,
            UR => DL,
            DL => UR,
            DR => UL,
        }
    }
}

pub enum SnakeType {
    SinglePlayer,
    Player1,
    Player2,
}

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Crashed,
}

pub struct Snake {
    snake_type: SnakeType,

    pub body: Vec<Hex>,
    growing: usize,
    dir: Dir,
    pub game_dim: HexPos, // cached value

    pub state: SnakeState,

    pub ctrl: Controls,
    ctrl_queue: VecDeque<Dir>,
}

impl Snake {
    const CTRL_QUEUE_LIMIT: usize = 3;

    pub fn new(snake_type: SnakeType, dim: HexPos, start_pos: HexPos, ctrl: Controls) -> Self {
        let head = Hex {
            typ: Normal,
            pos: start_pos,
            teleported: None,
        };

        Self {
            snake_type,

            body: vec![head],
            growing: 15,
            dir: Dir::U,
            game_dim: dim,

            state: SnakeState::Living,

            ctrl,
            ctrl_queue: VecDeque::with_capacity(Self::CTRL_QUEUE_LIMIT),
        }
    }

    pub fn advance(&mut self) {
        self.pop_ctrl_queue();

        let mut new_head = Hex {
            typ: Normal,
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

    pub(in crate::game) fn draw_non_crash_points(
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color, Option<Dir>) -> GameResult,
        palette: &Palette,
    ) -> GameResult {
        use SnakeType::*;
        let snake_palette = match self.snake_type {
            SinglePlayer => &palette.single_player_snake,
            Player1 => &palette.player1_snake,
            Player2 => &palette.player2_snake,
        };

        // head to tail
        for (i, segment) in self.body.iter().enumerate() {
            let color = match segment.typ {
                Crashed => continue,
                Normal => {
                    snake_palette(i, self.body.len())
                }
                Eaten(_) => palette.eaten_color,
            };

            match segment.teleported {
                None => draw_cell(segment.h as usize, segment.v as usize, color, None)?,
                Some(dir) => {
                    draw_cell(segment.h as usize, segment.v as usize, color, Some(-dir))?;
                    draw_cell(segment.h as usize, segment.v as usize, palette.teleported_color, Some(dir))?;
                },
            }
        }

        Ok(())
    }

    pub(in crate::game) fn draw_crash_point(
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color, Option<Dir>) -> GameResult,
        palette: &Palette,
    ) -> GameResult {
        if self.body[0].typ == Crashed {
            draw_cell(
                self.body[0].h as usize,
                self.body[0].v as usize,
                palette.crash_color,
                None,
            )?
        }
        Ok(())
    }

    // -- control --
    pub fn key_pressed(&mut self, key: KeyCode) {
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
