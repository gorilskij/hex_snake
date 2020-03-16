use Dir::*;
use std::ops::Neg;
use super::hex::{Hex, HexType::*};
use crate::game::hex::hex_pos::HexPos;
use ggez::{Context, GameResult};
use ggez::graphics::{Color, Mesh, DrawMode};
use mint::Point2;
use crate::game::ctrl::Ctrl;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Dir { U, D, UL, UR, DL, DR }

impl Neg for Dir {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            U => D, D => U, UL => DR, UR => DL, DL => UR, DR => UL,
        }
    }
}


pub struct Snake {
    pub body: Vec<Hex>,
    growing: usize,
    dir: Dir,
    pub(crate) dim: HexPos,

    pub ctrl: Ctrl,
}

impl Snake {
    pub fn new(dim: HexPos, offset: HexPos, ctrl: Ctrl) -> Self {
        let center = Hex { typ: Normal, pos: dim / 2 + offset };
        Self {
            body: vec![center],
            growing: 15,
            dir: Dir::U,
            dim,

            ctrl,
        }
    }

    pub fn grow(&mut self, amount: usize) {
        self.growing += amount
    }

    pub fn advance(&mut self) {
        let mut new_head = Hex {
            typ: Normal,
            pos: self.body[0].pos,
        };

        // todo make O(1)
        new_head.translate(self.dir, 1);
        if !new_head.is_in(self.dim) {
            // step back
            new_head.translate(self.dir, -1);

            while new_head.is_in(self.dim) {
                new_head.translate(self.dir, -1);
            }
            new_head.translate(self.dir, 1);
        }

        if self.growing > 0 {
            self.body.insert(0, new_head);
            self.growing -= 1;
        } else {
            self.body.rotate_right(1);
            self.body[0] = new_head;
        }
    }

    // ignore opposite direction
    pub fn set_direction_safe(&mut self, new_dir: Dir) {
        if self.dir != -new_dir { self.dir = new_dir }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        hexagon_points: &[Point2<f32>],
        draw_cell: fn(usize, usize, &Mesh, &mut Context, f32, f32, f32) -> GameResult,
        sl: f32,
        s: f32,
        c: f32,
    ) -> GameResult {
        // head to tail
        for (i, segment) in self.body.iter().rev().enumerate() {
            let color = match segment.typ {
                Normal => {
                    // [0.5, 1]
                    let drk = (1. - i as f32 / self.body.len() as f32) / 2.;
                    Color { r: drk, b: drk, g: drk, a: 1. }
                }
                Crashed => Color { r: 1., b: 0.5, g: 0., a: 1. },
                Eaten => Color { r: 0., b: 0.5, g: 1., a: 1. },
                _ => panic!(),
            };

            let segment_fill = Mesh::new_polyline(
                ctx, DrawMode::fill(),
                hexagon_points, color)?;

            draw_cell(segment.h as usize, segment.v as usize,
                      &segment_fill, ctx, sl, s, c)?
        }

        Ok(())
    }
}