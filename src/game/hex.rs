pub use hex_pos::HexPos;
use std::ops::{Deref, DerefMut};

mod hex_pos {
    use crate::game::snake::Dir;
    use num_integer::Integer;
    use rand::Rng;
    use std::fmt::{Debug, Error, Formatter};
    use Dir::*;

    #[derive(Eq, PartialEq, Copy, Clone, Div, Add)]
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
        pub fn random_in<R: Rng>(dim: Self, rng: &mut R) -> Self {
            Self {
                h: rng.gen_range(0, dim.h),
                v: rng.gen_range(0, dim.v),
            }
        }

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

    Apple,
}

#[derive(Copy, Clone, Debug)]
pub struct Hex {
    pub typ: HexType,
    pub pos: HexPos,
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
