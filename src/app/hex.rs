use std::ops::{Deref, DerefMut};

pub use dir::Dir;
pub use hex_pos::HexPos;

mod dir {
    use std::ops::Neg;

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
}

mod hex_pos {
    use std::fmt::{Debug, Error, Formatter};

    use num_integer::Integer;

    use Dir::*;

    use super::dir::Dir;

    #[derive(Eq, PartialEq, Copy, Clone, Div, Add, Hash)]
    pub struct HexPos {
        pub h: isize,
        pub v: isize,
    }

    impl Debug for HexPos {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            write!(f, "<{}, {}>", self.h, self.v)
        }
    }

    impl HexPos {
        // translates h/v with special treatment for v
        pub fn translate(&mut self, dir: Dir, dist: isize) {
            if dist < 0 {
                self.translate(-dir, -dist);
                return;
            }

            let half = (dist as f64 / 2.).ceil() as isize;
            match dir {
                U => self.v -= dist,
                D => self.v += dist,
                UL => {
                    self.h -= dist;
                    self.v -= half;
                    if self.h.is_even() {
                        self.v += 1
                    }
                }
                UR => {
                    self.h += dist;
                    self.v -= half;
                    if self.h.is_even() {
                        self.v += 1
                    }
                }
                DL => {
                    self.h -= dist;
                    self.v += half;
                    if self.h.is_odd() {
                        self.v -= 1
                    }
                }
                DR => {
                    self.h += dist;
                    self.v += half;
                    if self.h.is_odd() {
                        self.v -= 1
                    }
                }
            }
        }

        pub fn step_and_teleport(&mut self, dir: Dir, board_dim: HexPos) {
            self.translate(dir, 1);
            if !self.is_in(board_dim) {
                // find reappearance point
                self.translate(dir, -1);
                while self.is_in(board_dim) {
                    self.translate(dir, -1);
                }
                self.translate(dir, 1);
            }
        }

        // checks if between (0,0) and dim
        pub fn is_in(self, dim: HexPos) -> bool {
            self.h >= 0 && self.h < dim.h && self.v >= 0 && self.v < dim.v
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum HexType {
    Normal,
    Crashed,
    Eaten(u8),
}

#[derive(Copy, Clone, Debug)]
pub struct Hex {
    pub typ: HexType,
    pub pos: HexPos,
    pub teleported: Option<Dir>,
}

impl Deref for Hex {
    type Target = HexPos;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}

impl DerefMut for Hex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pos
    }
}
