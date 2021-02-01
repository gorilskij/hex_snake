use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use rand::Rng;
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
        match self {
            U => D,
            D => U,
            UL => DR,
            UR => DL,
            DL => UR,
            DR => UL,
        }
        // hypothetically: ((self as u8 + 3) % 6) as Dir
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
        Self::from(self as u8 + 6 - (rhs % 6))
    }
}

impl SubAssign<u8> for Dir {
    fn sub_assign(&mut self, rhs: u8) {
        *self = *self - rhs;
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TurnDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TurnType {
    Straight,
    Blunt(TurnDirection),
    Sharp(TurnDirection),
}

impl Dir {
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

    // turn: self => other
    pub fn turn_type(self, other: Self) -> TurnType {
        use TurnDirection::*;
        use TurnType::*;

        let mut dir = self;
        let mut clockwise_distance = 0;
        while dir != other {
            clockwise_distance += 1;
            dir += 1;
        }

        match clockwise_distance {
            1 => Sharp(Clockwise),
            5 => Sharp(CounterClockwise),
            2 => Blunt(Clockwise),
            4 => Blunt(CounterClockwise),
            3 => Straight,
            _ => panic!("impossible turn {:?} => {:?}", self, other),
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        Self::from(rng.gen_range(0, 6))
    }

    pub fn clockwise_angle_from_u(self) -> f32 {
        use std::f32::consts::*;
        match self {
            U => 0.,
            UR => FRAC_PI_3,
            DR => 2. * FRAC_PI_3,
            D => 3. * FRAC_PI_3,
            DL => 4. * FRAC_PI_3,
            UL => 5. * FRAC_PI_3,
        }
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
