use std::fmt::{Debug, Formatter, Error};
use crate::game::snake::Dir;
use Dir::*;
use std::ops::{Rem, Add};

trait IsEven where Self: Sized {
    fn is_even(self) -> bool;
    fn is_odd(self) -> bool {
        !self.is_even()
    }
}

impl<T: Copy + Rem<T, Output=T> + Add<T, Output=T> + PartialEq + From<i8>> IsEven for T {
    fn is_even(self) -> bool {
        let two = 2.into();
        ((self % two) + two) % two == 0.into()
    }
}


#[derive(PartialEq, Copy, Clone, Div)]
pub struct HexGridPoint {
    pub h: isize,
    pub v: isize,
}

impl Debug for HexGridPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<{}, {}>", self.h, self.v)
    }
}

impl HexGridPoint {
    // translates h/v with special treatment for v
    pub fn translate(&mut self, dir: Dir, dist: isize) {
        if dist < 0 {
            self.translate(dir.opposite(), -dist);
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
    pub fn is_in(self, dim: HexGridPoint) -> bool {
        self.h >= 0
            && self.h < dim.h
            && self.v >= 0
            && self.v < dim.v
    }
}