use crate::game::hex_grid_point::HexGridPoint;
use Dir::*;

#[derive(Copy, Clone)]
pub enum Dir { U, D, UL, UR, DL, DR }

impl Dir {
    pub fn opposite(self) -> Self {
        match self {
            U => D, D => U, UL => DR, UR => DL, DL => UR, DR => UL,
        }
    }
}


pub struct Snake {
    pub body: Vec<HexGridPoint>,
    dir: Dir,
    dim: HexGridPoint,
}

impl Snake {
    pub fn new(dim: HexGridPoint) -> Self {
        let center = dim / 2;
        let mut body = vec![];
        for offset in 0..4 {
            body.push(HexGridPoint { h: center.h, v: center.v + offset })
        }

        Self {
            body,
            dir: Dir::U,
            dim,
        }
    }

    pub fn advance(&mut self) {
        let mut new_head = self.body[0];

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

        self.body.rotate_right(1);
        self.body[0] = new_head;
    }

    pub fn set_direction(&mut self, new_direction: Dir) {
        self.dir = new_direction
    }
}