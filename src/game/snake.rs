use crate::game::hex_grid_point::HexGridPoint;

pub enum Dir {
    Up, RightUp, RightDown, Down, LeftUp, LeftDown,
}

pub struct Snake {
    body: Vec<HexGridPoint>,
    dir: Dir,
}

impl Snake {
    pub fn new(head_h: usize, head_v: usize) -> Self {
        let body = (head_v..head_v + 5)
            .map(|v| HexGridPoint { h: head_h, v })
            .collect();

        Self {
            body,
            dir: Dir::Up,
        }
    }

    pub fn is_at(&self, point: HexGridPoint) -> bool {
        self.body.iter()
            .any(|&p| p == point)
    }

    pub fn advance(&mut self) {
        let mut head = self.body[0];
        self.body.rotate_right(1);

        use Dir::*;
        match self.dir {
            Up => head.v -= 1,
            RightUp => {
                if head.h % 2 == 0 {
                    head.v -= 1;
                }
                head.h += 1;
            },
            RightDown => {
                if head.h % 2 != 0 {
                    head.v += 1;
                }
                head.h +=1;
            },
            Down => head.v += 1,
            LeftUp => {
                if head.h % 2 == 0 {
                    head.v -= 1;
                }
                head.h -= 1;
            },
            LeftDown => {
                if head.h % 2 != 0 {
                    head.v += 1;
                }
                head.h -= 1;
            },
        }

        self.body[0] = head;
    }

    pub fn set_direction(&mut self, new_direction: Dir) {
        self.dir = new_direction
    }
}