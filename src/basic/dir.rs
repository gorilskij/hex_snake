use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use itertools::Itertools;

use crate::basic::angle_distance;
use rand::Rng;
use std::cmp::Ordering;
use std::f32::consts::TAU;
use Dir::*;

// defined in clockwise order starting at U
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Dir {
    U = 0,
    Ur = 1,
    Dr = 2,
    D = 3,
    Dl = 4,
    Ul = 5,
}

impl From<u8> for Dir {
    fn from(num: u8) -> Self {
        // SAFETY: (num % 6) is between 0 and 5
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

impl Add<Self> for Dir {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self + rhs as u8
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

// U is the smallest, directions get bigger clockwise, Ul is the largest
// Ul > U
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
    let test_plus = [(U, 1, Ur), (U, 2, Dr), (Dr, 3, Ul), (D, 6, D)];

    for &(start, add, expect) in &test_plus {
        assert_eq!(start + add, expect);
    }

    let test_minus = [(U, 1, Ul), (U, 2, Dl), (Dr, 3, Ul), (D, 6, D)];

    for &(start, sub, expect) in &test_minus {
        assert_eq!(start - sub, expect);
    }
}

pub enum Axis {
    UD,   // |
    UlDr, // \
    UrDl, // /
}

impl Dir {
    // angles around the unit circle
    pub const ANGLES: [(Dir, f32); 6] = [
        (U, 3. / 12. * TAU),
        (Ur, 1. / 12. * TAU),
        (Dr, 11. / 12. * TAU),
        (D, 9. / 12. * TAU),
        (Dl, 7. / 12. * TAU),
        (Ul, 5. / 12. * TAU),
    ];

    /// Return all `Dir`s sorted by how close they are to the given angle
    pub fn closest_to_angle(angle: f32) -> Vec<Self> {
        Self::ANGLES
            .into_iter()
            .map(|(dir, ang)| (dir, ang, angle_distance(angle, ang)))
            .sorted_by(|(_, _, dist1), (_, _, dist2)| dist1.partial_cmp(dist2).unwrap())
            .map(|(dir, _, _)| dir)
            .collect()
    }

    // clockwise order starting from U
    pub fn iter() -> impl Iterator<Item = Self> {
        [U, Ur, Dr, D, Dl, Ul].iter().copied()
    }

    pub fn iter_from(start: Self) -> impl Iterator<Item = Self> {
        Self::iter().map(move |dir| dir + start)
    }

    pub fn axis(self) -> Axis {
        use Axis::*;

        match self {
            U | D => UD,
            Ul | Dr => UlDr,
            Ur | Dl => UrDl,
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
        const C_UL: &[Dir] = &[Dl, U];
        const C_U: &[Dir] = &[Ul, Ur];
        const C_UR: &[Dir] = &[U, Dr];
        const C_DR: &[Dir] = &[Ur, D];
        const C_D: &[Dir] = &[Dr, Dl];
        const C_DL: &[Dir] = &[D, Ul];
        match self {
            Ul => C_UL,
            U => C_U,
            Ur => C_UR,
            Dr => C_DR,
            D => C_D,
            Dl => C_DL,
        }
    }

    pub fn sharp_turns(self) -> &'static [Self] {
        const C_UL: &[Dir] = &[D, Ur];
        const C_U: &[Dir] = &[Dl, Dr];
        const C_UR: &[Dir] = &[Ul, D];
        const C_DR: &[Dir] = &[U, Dl];
        const C_D: &[Dir] = &[Ur, Ul];
        const C_DL: &[Dir] = &[Dr, U];
        match self {
            Ul => C_UL,
            U => C_U,
            Ur => C_UR,
            Dr => C_DR,
            D => C_D,
            Dl => C_DL,
        }
    }
}

#[test]
fn test_clockwise_distance_to() {
    use Dir::*;
    for (from, to, clockwise_dist) in [
        (U, U, 0),
        (U, Ur, 1),
        (U, Dr, 2),
        (U, D, 3),
        (U, Dl, 4),
        (U, Ul, 5),
        (Ur, Dr, 1),
        (Dr, Ul, 3),
        (Ul, Dr, 3),
        (Dl, U, 2),
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
