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
        // clockwise order starting from U
        pub fn iter() -> impl Iterator<Item = Self> {
            [U, UR, DR, D, DL, UL].iter().copied()
        }

        pub fn clockwise(self) -> Self {
            match self {
                U => UR,
                UR => DR,
                DR => D,
                D => DL,
                DL => UL,
                UL => U,
            }
        }

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

        // TODO: replace this with manhattan distance
        // approximate straight-line distance in units of side length
        pub fn distance_to(self, HexPos { h, v }: HexPos) -> usize {
            let dh = (self.h - h).abs() as f32;
            let dv = (self.v - v).abs() as f32;
            let CellDim { side, sin, cos } = CellDim::from(1.);
            (dh / (side + cos) + dv / (2. * sin)) as usize
        }

        // all cells within a manhattan distance of radius (including self)
        // guarantees no duplicates, not sorted
        pub fn neighborhood(self, radius: usize) -> Vec<Self> {
            let mut neighborhood = vec![self];
            for dir in Dir::iter() {
                for r in 1..radius {
                    let branch = self.translate(dir, r);
                    neighborhood.push(branch);
                    let dir2 = dir.clockwise();
                    for r2 in 1..(radius - r) {
                        neighborhood.push(branch.translate(dir2, r2))
                    }
                }
            }

            // check that no duplicates were introduced
            // let len = neighborhood.len();
            // neighborhood.sort()
            // neighborhood.dedup();
            // assert_eq!(neighborhood.len(), len);

            neighborhood
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
        // TODO: figure out O(1) translation for longer distances
        fn translate_one_in_place(&mut self, dir: Dir) {
            match dir {
                U => self.v -= 1,
                D => self.v += 1,
                UL => {
                    self.h -= 1;
                    self.v -= 1;
                    if self.h.is_even() {
                        self.v += 1
                    }
                }
                UR => {
                    self.h += 1;
                    self.v -= 1;
                    if self.h.is_even() {
                        self.v += 1
                    }
                }
                DL => {
                    self.h -= 1;
                    self.v += 1;
                    if self.h.is_odd() {
                        self.v -= 1
                    }
                }
                DR => {
                    self.h += 1;
                    self.v += 1;
                    if self.h.is_odd() {
                        self.v -= 1
                    }
                }
            }
        }

        #[must_use]
        pub fn translate(self, dir: Dir, dist: usize) -> Self {
            let mut new_pos = self;
            for _ in 0..dist {
                new_pos.translate_one_in_place(dir);
            }
            new_pos
        }
        
        // broken
        // // translates h/v with special treatment for v
        // #[must_use]
        // pub fn translate(self, dir: Dir, dist: isize) -> Self {
        //     if dist < 0 {
        //         return self.translate(-dir, -dist);
        //     }
        // 
        //     let mut new_pos = self;
        //     let half = (dist as f64 / 2.).ceil() as isize;
        //     match dir {
        //         U => new_pos.v -= dist,
        //         D => new_pos.v += dist,
        //         UL => {
        //             new_pos.h -= dist;
        //             new_pos.v -= half;
        //             if new_pos.h.is_even() {
        //                 new_pos.v += 1
        //             }
        //         }
        //         UR => {
        //             new_pos.h += dist;
        //             new_pos.v -= half;
        //             if new_pos.h.is_even() {
        //                 new_pos.v += 1
        //             }
        //         }
        //         DL => {
        //             new_pos.h -= dist;
        //             new_pos.v += half;
        //             if new_pos.h.is_odd() {
        //                 new_pos.v -= 1
        //             }
        //         }
        //         DR => {
        //             new_pos.h += dist;
        //             new_pos.v += half;
        //             if new_pos.h.is_odd() {
        //                 new_pos.v -= 1
        //             }
        //         }
        //     }
        // 
        //     new_pos
        // }

        // wraps around board edges
        #[must_use]
        pub fn wrapping_translate(self, dir: Dir, dist: usize, board_dim: HexDim) -> Self {
            // TODO: make O(1)
            //  at the moment this just moves the head back until the last cell that's still in the map
            //  this could be done as a single calculation
            // TODO: generalize to any number of steps

            let mut new_pos = self.translate(dir, dist);
            if !board_dim.contains(new_pos) {
                // find reappearance point
                while !board_dim.contains(new_pos) {
                    new_pos = new_pos.translate(-dir, 1);
                }
                while board_dim.contains(new_pos) {
                    new_pos = new_pos.translate(-dir, 1);
                }
                new_pos.translate(dir, 1)
            } else {
                new_pos
            }
        }

        // checks if between (0,0) and dim
        pub fn contains(self, pos: Self) -> bool {
            (0..self.h).contains(&pos.h) && (0..self.v).contains(&pos.v)
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