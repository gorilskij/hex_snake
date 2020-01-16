use crate::game::hex_grid_point::HexGridPoint;
use std::cmp::min;

pub enum Dir {
    Up, RightUp, RightDown, Down, LeftUp, LeftDown,
}

pub struct Snake {
    body: Vec<HexGridPoint>,
    dir: Dir,
    width: usize,
    height: usize,
}

impl Snake {
    pub fn new(width: usize, height: usize) -> Self {
        let (head_h, head_v) = (width / 2, height / 2);
        let body = (head_v..head_v + 5)
            .map(|v| HexGridPoint { h: head_h, v })
            .collect();

        Self {
            body,
            dir: Dir::Up,
            width,
            height,
        }
    }

    pub fn is_at(&self, point: HexGridPoint) -> bool {
        self.body.iter()
            .any(|&p| p == point)
    }

    pub fn advance(&mut self) {
        let mut head = self.body[0];
        self.body.rotate_right(1);

        let move_head = |head: &mut HexGridPoint, delta_h: isize, delta_v: isize| {
            let delta_min = min(delta_h.abs(), delta_v.abs() * 2);
            head.h = (head.h as isize + if delta_h < 0 { -delta_min } else { delta_min }) as usize;
            head.v = (head.v as isize + if delta_v < 0 { -delta_min } else { delta_min } / 2) as usize;
        };
        use Dir::*;
        match self.dir {
            Up => head.v = if head.v == 0 { self.height - 1 } else { head.v - 1 },
            RightUp => if head.h == self.width || head.v == 0 && head.h % 2 == 0 {
                let delta_h = -(head.h as isize) - 1;
                let delta_v =  (self.height - head.v - 1) as isize;
                move_head(&mut head, delta_h, delta_v);
            } else {
                if head.h % 2 == 0 { head.v -= 1 }
                head.h += 1
            },
            RightDown => if head.h == self.width - 1 || head.v == self.height - 1 {
                let delta_h = -(head.h as isize);
                let delta_v =  -(head.v as isize);
                move_head(&mut head, delta_h, delta_v);
            } else {
                if head.h % 2 != 0 { head.v += 1 }
                head.h += 1
            },
            Down => if head.v == self.height - 1 {
                head.v = 0
            } else {
                head.v += 1
            },
            LeftUp => if head.h == 0 || head.v == 0 {
                let delta_h = (self.width - head.h - 1) as isize;
                let delta_v =  (self.height - head.v - 1) as isize;
                move_head(&mut head, delta_h, delta_v);
            } else {
                if head.h % 2 == 0 { head.v -= 1 }
                head.h -= 1;
            },
            LeftDown => if head.h == 0 || head.v == self.height - 1 {
                let delta_h = (self.width - head.h - 1) as isize;
                let delta_v =  -(head.v as isize);
                move_head(&mut head, delta_h, delta_v);
            } else {
                if head.h % 2 != 0 { head.v += 1; }
                head.h -= 1;
            },
        }

//        head.h = (head.h + self.width) % self.width;
//        head.v = (head.v + self.height) % self.height;

        self.body[0] = head;
    }

    pub fn set_direction(&mut self, new_direction: Dir) {
        self.dir = new_direction
    }
}