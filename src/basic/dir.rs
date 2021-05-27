use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use rand::Rng;
use std::{cmp::Ordering, f32::consts::PI};
use Dir::*;

// defined in clockwise order starting at U
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Dir {
    U = 0,
    UR = 1,
    DR = 2,
    D = 3,
    DL = 4,
    UL = 5,
}

impl From<u8> for Dir {
    fn from(num: u8) -> Self {
        unsafe { std::mem::transmute(num % 6) }
    }
}

impl Neg for Dir {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self + 3
    }
}

impl Add<u8> for Dir {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self::from(self as u8 + rhs)
    }
}

impl AddAssign<u8> for Dir {
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

impl Sub<u8> for Dir {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        self + (6 - (rhs % 6))
    }
}

impl SubAssign<u8> for Dir {
    fn sub_assign(&mut self, rhs: u8) {
        *self = *self - rhs;
    }
}

// U is the smallest, directions get bigger clockwise, UL is the largest
// UL > U
impl Ord for Dir {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for Dir {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_dir_math() {
    let test_plus = [(U, 1, UR), (U, 2, DR), (DR, 3, UL), (D, 6, D)];

    for &(start, add, expect) in &test_plus {
        assert_eq!(start + add, expect);
    }

    let test_minus = [(U, 1, UL), (U, 2, DL), (DR, 3, UL), (D, 6, D)];

    for &(start, sub, expect) in &test_minus {
        assert_eq!(start - sub, expect);
    }
}

pub enum Axis {
    UD,   // |
    ULDR, // \
    URDL, // /
}

impl Dir {
    // angles around the unit circle
    pub const ANGLES: [(Dir, f32); 6] = [
        (U, 3. / 6. * PI),
        (UR, 1. / 6. * PI),
        (DR, 11. / 6. * PI),
        (D, 9. / 6. * PI),
        (DL, 7. / 6. * PI),
        (UL, 5. / 6. * PI),
    ];

    // clockwise order starting from U
    pub fn iter() -> impl Iterator<Item = Self> {
        [U, UR, DR, D, DL, UL].iter().copied()
    }

    pub fn axis(self) -> Axis {
        use Axis::*;

        match self {
            U | D => UD,
            UL | DR => ULDR,
            UR | DL => URDL,
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        Self::from(rng.gen_range(0, 6))
    }

    /// Clockwise angle from self to other in units of 60°
    pub fn clockwise_distance_to(self, other: Self) -> u8 {
        (other as u8 + 6 - self as u8) % 6
    }

    /// Similar to `clockwise_distance_to` but returns an angle in radians
    pub fn clockwise_angle_to(self, other: Self) -> f32 {
        self.clockwise_distance_to(other) as f32 * std::f32::consts::FRAC_PI_3
    }

    pub fn blunt_turns(self) -> &'static [Self] {
        const C_UL: &[Dir] = &[DL, U];
        const C_U: &[Dir] = &[UL, UR];
        const C_UR: &[Dir] = &[U, DR];
        const C_DR: &[Dir] = &[UR, D];
        const C_D: &[Dir] = &[DR, DL];
        const C_DL: &[Dir] = &[D, UL];
        match self {
            UL => C_UL,
            U => C_U,
            UR => C_UR,
            DR => C_DR,
            D => C_D,
            DL => C_DL,
        }
    }

    pub fn sharp_turns(self) -> &'static [Self] {
        const C_UL: &[Dir] = &[D, UR];
        const C_U: &[Dir] = &[DL, DR];
        const C_UR: &[Dir] = &[UL, D];
        const C_DR: &[Dir] = &[U, DL];
        const C_D: &[Dir] = &[UR, UL];
        const C_DL: &[Dir] = &[DR, U];
        match self {
            UL => C_UL,
            U => C_U,
            UR => C_UR,
            DR => C_DR,
            D => C_D,
            DL => C_DL,
        }
    }
}

#[test]
fn test_clockwise_distance_to() {
    use Dir::*;
    for (from, to, clockwise_dist) in [
        (U, U, 0),
        (U, UR, 1),
        (U, DR, 2),
        (U, D, 3),
        (U, DL, 4),
        (U, UL, 5),
        (UR, DR, 1),
        (DR, UL, 3),
        (UL, DR, 3),
        (DL, U, 2),
    ] {
        assert_eq!(
            from.clockwise_distance_to(to),
            clockwise_dist,
            "{:?} => {:?}",
            from,
            to
        )
    }
}
