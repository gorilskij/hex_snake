use std::ops::{Deref, DerefMut};
pub use hex_pos::{HexPos, IsEven};

mod hex_pos {
    use std::fmt::{Debug, Formatter, Error};
    use rand::Rng;
    use crate::game::snake::Dir;
    use Dir::*;

    // todo move to a better place
    pub trait IsEven where Self: Sized {
        fn is_even(self) -> bool;
        fn is_odd(self) -> bool {
            !self.is_even()
        }
    }

    macro_rules! impl_is_even {
        ($type:ty) => {
            impl IsEven for $type {
                fn is_even(self) -> bool {
                    ((self % 2 as $type) + 2 as $type) % 2 as $type == 0 as $type
                }
            }
        };
    }

    impl_is_even!(isize);
    impl_is_even!(usize);

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
                    if self.h.is_even() { self.v += 1 }
                }
                UR => {
                    self.h += dist;
                    self.v -= half;
                    if self.h.is_even() { self.v += 1 }
                }
                DL => {
                    self.h -= dist;
                    self.v += half;
                    if self.h.is_odd() { self.v -= 1 }
                }
                DR => {
                    self.h += dist;
                    self.v += half;
                    if self.h.is_odd() { self.v -= 1 }
                }
            }
        }

        // checks if between (0,0) and dim
        pub fn is_in(self, dim: HexPos) -> bool {
            self.h >= 0
                && self.h < dim.h
                && self.v >= 0
                && self.v < dim.v
        }
    }
}


#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum HexType {
    Normal,
    Crashed,
    Eaten,

    Apple,
}

#[derive(Copy, Clone, Debug)] // ?
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