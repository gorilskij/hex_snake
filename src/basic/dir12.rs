use crate::basic::Dir;
use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub, SubAssign},
};
use Dir::*;
use Dir12::*;

// includes directions between the usual 6
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Dir12 {
    Single(Dir),
    // the two directions must be in clockwise order
    // except if it's (UL, U) (clockwise but not Ord)
    // this variant shouldn't be constructed manually
    Combined(Dir, Dir),
}

impl Dir12 {
    //     const ORDER: &'static [Dir12] = &[
    //         Dir12::Single(Dir::U),
    //         Dir12::Combined(Dir::U, Dir::UR),
    //         Dir12::Single(Dir::UR),
    //         Dir12::Combined(Dir::UR, Dir::DR),
    //         Dir12::Single(Dir::DR),
    //         Dir12::Combined(Dir::DR, Dir::D),
    //         Dir12::Single(Dir::D),
    //         Dir12::Combined(Dir::D, Dir::DL),
    //         Dir12::Single(Dir::DL),
    //         Dir12::Combined(Dir::DL, Dir::UL),
    //         Dir12::Single(Dir::UL),
    //         Dir12::Combined(Dir::UL, Dir::U),
    //     ];

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Single(U),
            Combined(U, UR),
            Single(UR),
            Combined(UR, DR),
            Single(DR),
            Combined(DR, D),
            Single(D),
            Combined(D, DL),
            Single(DL),
            Combined(DL, UL),
            Single(UL),
            Combined(UL, U),
        ]
        .iter()
        .copied()
    }
}

impl Ord for Dir12 {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Single(dir1) => match other {
                Single(dir2) => dir1.cmp(dir2),
                Combined(dir2, _) => {
                    if dir1 <= dir2 {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
            },
            Combined(dir1, _) => match other {
                Single(dir2) => {
                    if dir1 >= dir2 {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
                Combined(dir2, _) => dir1.cmp(dir2),
            },
        }
    }
}

impl PartialOrd for Dir12 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_dir12_ord() {
    for (a, b) in Dir12::iter().zip(Dir12::iter().skip(1)) {
        assert!(a < b, "failed {:?} < {:?}", a, b);
    }
}

// unlike Dir, this goes in steps of 1/12
impl Add<u8> for Dir12 {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        if rhs % 2 == 0 {
            let rhs = rhs / 2; // steps of 1/6
            match self {
                Single(dir) => Single(dir + rhs),
                Combined(dir1, dir2) => Combined(dir1 + rhs, dir2 + rhs),
            }
        } else {
            match self {
                Single(dir) => Combined(dir + rhs - 1, dir + rhs),
                Combined(dir1, _) => Single(dir1 + rhs),
            }
        }
    }
}

impl AddAssign<u8> for Dir12 {
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

impl Sub<u8> for Dir12 {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        if rhs % 2 == 0 {
            let rhs = rhs / 2; // steps of 1/6
            match self {
                Single(dir) => Single(dir - rhs),
                Combined(dir1, dir2) => Combined(dir1 - rhs, dir2 - rhs),
            }
        } else {
            match self {
                Single(dir) => Combined(dir - rhs, dir - rhs + 1),
                Combined(_, dir2) => Single(dir2 - rhs),
            }
        }
    }
}

impl SubAssign<u8> for Dir12 {
    fn sub_assign(&mut self, rhs: u8) {
        *self = *self - rhs;
    }
}

#[test]
fn test_dir12_math() {
    for (a, b) in Dir12::iter().zip(Dir12::iter().skip(1)) {
        assert_eq!(a + 1, b, "failed {:?} + 1 == {:?}", a, b);
        assert_eq!(a, b - 1, "failed {:?} == {:?} - 1", a, b);
    }

    for (a, b) in Dir12::iter().zip(Dir12::iter().skip(2)) {
        assert_eq!(a + 2, b, "failed {:?} + 2 == {:?}", a, b);
        assert_eq!(a, b - 2, "failed {:?} == {:?} - 2", a, b);
    }
}
