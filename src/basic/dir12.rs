use crate::basic::Dir;
use std::{
    cmp::Ordering,
    f32::consts::PI,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
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
    // angles of directions, clockwise from U
    pub const ANGLES: [(Dir12, f32); 12] = [
        (Single(U), 3. / 6. * PI),
        (Combined(U, UR), 2. / 6. * PI),
        (Single(UR), 1. / 6. * PI),
        (Combined(UR, DR), 0. / 6. * PI),
        (Single(DR), 11. / 6. * PI),
        (Combined(DR, D), 10. / 6. * PI),
        (Single(D), 9. / 6. * PI),
        (Combined(D, DL), 8. / 6. * PI),
        (Single(DL), 7. / 6. * PI),
        (Combined(DL, UL), 6. / 6. * PI),
        (Single(UL), 5. / 6. * PI),
        (Combined(UL, U), 4. / 6. * PI),
    ];

    // clockwise from U
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

    // flip_flop_state is meant to change on every frame
    pub fn to_dir(self, flip_flop_state: bool) -> Dir {
        match self {
            Single(dir) => dir,
            Combined(a, b) => {
                if flip_flop_state {
                    b
                } else {
                    a
                }
            }
        }
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

impl Neg for Dir12 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self + 6
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

    for (a, b) in Dir12::iter().zip(Dir12::iter().skip(6).chain(Dir12::iter())) {
        assert_eq!(a, -b, "failed {:?} == -{:?}", a, b);
        assert_eq!(-a, b, "failed -{:?} == {:?}", a, b);
    }
}
