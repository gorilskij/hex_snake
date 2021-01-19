use std::ops::{Deref, DerefMut};

pub use dir::Dir;
pub use hex_pos::{HexDim, HexPos};

mod dir {
    use std::ops::Neg;

    use rand::Rng;
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

    impl Dir {
        pub fn random(rng: &mut impl Rng) -> Self {
            match rng.gen_range(0, 6) {
                0 => U,
                1 => D,
                2 => UL,
                3 => UR,
                4 => DL,
                5 => DR,
                _ => unreachable!(),
            }
        }
    }
}

mod hex_pos {
    use super::dir::Dir;
    use crate::app::game::CellDim;
    use ggez::mint::Point2;
    use num_integer::Integer;
    use std::{
        cmp::Ordering,
        fmt::{Debug, Error, Formatter},
    };
    use Dir::*;

    #[derive(Eq, PartialEq, Copy, Clone, Div, Add, Hash)]
    pub struct HexPos {
        pub h: isize,
        pub v: isize,
    }

    pub type HexDim = HexPos;

    impl HexPos {
        pub fn to_point(self, CellDim { side, sin, cos }: CellDim) -> Point2<f32> {
            let Self { h, v } = self;
            Point2 {
                x: h as f32 * (side + cos),
                y: v as f32 * 2. * sin + if h % 2 == 0 { 0. } else { sin },
            }
        }
    }

    impl Debug for HexPos {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            write!(f, "<{}, {}>", self.h, self.v)
        }
    }

    impl PartialOrd for HexPos {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for HexPos {
        fn cmp(&self, other: &Self) -> Ordering {
            match self.v.cmp(&other.v) {
                Ordering::Equal => self.h.cmp(&other.h),
                ord => ord,
            }
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

        pub fn step_and_teleport(&mut self, dir: Dir, board_dim: HexDim) {
            // todo make O(1)
            //  at the moment this just moves the head back until the last cell that's still in the map
            //  this could be done as a single calculation

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
        pub fn is_in(self, dim: HexDim) -> bool {
            (0..dim.h).contains(&self.h) && (0..dim.v).contains(&self.v)
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum HexType {
    Normal,
    Crashed,
    Eaten(u32),
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
