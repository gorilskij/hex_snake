use std::fmt::{Debug, Formatter, Error};

#[derive(PartialEq, Copy, Clone)]
pub struct HexGridPoint {
    pub h: usize,
    pub v: usize,
}

impl Debug for HexGridPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<{}, {}>", self.h, self.v)
    }
}

impl HexGridPoint {
    // translates x/y with special treatment for y (v)
    pub fn translate(self, h: isize, v: isize) -> Self {
        Self {
            h: (self.h as isize + h) as usize,
            v: (self.v as isize + v / 2) as usize,
        }
    }
}