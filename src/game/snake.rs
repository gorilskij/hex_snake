use Dir::*;
use std::ops::Neg;
use super::hex::{Hex, HexType::*};
use crate::game::hex::hex_pos::HexPos;

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
    dim: HexPos,
}

impl Snake {
    pub fn new(dim: HexPos) -> Self {
        let center = Hex { typ: Normal, pos: dim / 2 };
        Self {
            body: vec![center],
            growing: 15,
            dir: Dir::U,
            dim,
        }
    }

    pub fn head(&self) -> Hex {
        self.body[0]
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

    pub fn crashed(&self) -> bool {
        self.body[1..].iter().any(|b| b.pos == self.body[0].pos)
    }

    // ignore opposite direction
    pub fn set_direction_safe(&mut self, new_dir: Dir) {
        if self.dir != -new_dir { self.dir = new_dir }
    }
}