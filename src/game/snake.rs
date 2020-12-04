use super::hex::{Hex, HexPos, HexType::*};
use crate::game::{ctrl::Ctrl, hex::HexType, theme::Palette};
use ggez::{event::KeyCode, graphics::Color, GameResult};
use std::{collections::VecDeque, ops::Neg};
use Dir::*;

#[derive(Copy, Clone, Eq, PartialEq)]
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

#[derive(Eq, PartialEq)]
pub enum SnakeState {
    Living,
    Crashed,
}

pub struct Snake {
    pub body: Vec<Hex>,
    growing: usize,
    dir: Dir,
    pub game_dim: HexPos, // cached value

    pub state: SnakeState,

    pub ctrl: Ctrl,
    ctrl_queue: VecDeque<Dir>,
}

impl Snake {
    const CTRL_QUEUE_LIMIT: usize = 3;

    pub fn new(dim: HexPos, offset: HexPos, ctrl: Ctrl) -> Self {
        let center = Hex {
            typ: Normal,
            pos: dim / 2 + offset,
        };
        Self {
            body: vec![center],
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
        if let HexType::Eaten(amount, _) = &mut self.body[body_last].typ {
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
            self.body[0].typ = match self.body[0].typ {
                Normal => HexType::Teleported,
                Eaten(n, _) => Eaten(n, true),
                t => t,
            };
            self.body[1].typ = match self.body[1].typ {
                Normal => HexType::Teleported,
                Eaten(n, _) => Eaten(n, true),
                t => t,
            };
        }
    }

    pub(in crate::game) fn draw_non_crash_points(
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color) -> GameResult,
        palette: &Palette,
    ) -> GameResult {
        let head = palette.snake_head_color;
        let tail = palette.snake_tail_color;

        // head to tail
        for (i, segment) in self.body.iter().enumerate() {
            let color = match segment.typ {
                Crashed => continue,
                Normal => {
                    // darkness of the color, range: [0.5, 1]
                    // let darkness = (1. - i as f32 / self.body.len() as f32) / 2.;
                    let head_color_ratio = 1. - i as f32 / (self.body.len() - 1) as f32;
                    let tail_color_ratio = 1. - head_color_ratio;
                    Color {
                        r: head_color_ratio * head.r + tail_color_ratio * tail.r,
                        g: head_color_ratio * head.g + tail_color_ratio * tail.g,
                        b: head_color_ratio * head.b + tail_color_ratio * tail.b,
                        a: 1.,
                    }
                }
                // todo: include these in the palette
                Eaten(_, telepordted) => if telepordted {
                    Color {
                        r: 0.50,
                        g: 0.80,
                        b: 0.3,
                        a: 1.,
                    }
                } else {
                    Color {
                        r: 0.,
                        g: 1.,
                        b: 0.5,
                        a: 1.,
                    }
                },
                Teleported => Color {
                    r: 0.96,
                    g: 0.75,
                    b: 0.26,
                    a: 1.,
                },
            };

            draw_cell(segment.h as usize, segment.v as usize, color)?
        }

        Ok(())
    }

    pub(in crate::game) fn draw_crash_point(
        &self,
        draw_cell: &mut impl FnMut(usize, usize, Color) -> GameResult,
        palette: &Palette,
    ) -> GameResult {
        if self.body[0].typ == Crashed {
            draw_cell(
                self.body[0].h as usize,
                self.body[0].v as usize,
                palette.snake_crash_color,
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
