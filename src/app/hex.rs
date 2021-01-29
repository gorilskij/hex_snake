pub use dir::{Dir, TurnDirection, TurnType};
pub use hex_pos::{HexDim, HexPoint};

mod dir {
    use std::ops::Neg;

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

        pub fn next_clockwise(self) -> Self {
            match self {
                U => UR,
                UR => DR,
                DR => D,
                D => DL,
                DL => UL,
                UL => U,
            }
            // hypothetically: ((self as u8 + 1) % 6) as Dir
        }

        // turn: self => other
        pub fn turn_type(self, other: Self) -> TurnType {
            use TurnDirection::*;
            use TurnType::*;

            let mut dir = self;
            let mut clockwise_distance = 0;
            while dir != other {
                clockwise_distance += 1;
                dir = dir.next_clockwise();
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
}

mod hex_pos {
    use super::dir::{Axis, Dir};
    use crate::{app::game::CellDim, point::Point};
    use num_integer::Integer;
    use std::{
        cmp::Ordering,
        fmt::{Debug, Error, Formatter},
    };
    use Dir::*;

    #[derive(Eq, PartialEq, Copy, Clone, Div, Add, Hash)]
    pub struct HexPoint {
        pub h: isize,
        pub v: isize,
    }

    pub type HexDim = HexPoint;

    impl HexPoint {
        pub fn to_point(self, CellDim { side, sin, cos }: CellDim) -> Point {
            let Self { h, v } = self;
            Point {
                x: h as f32 * (side + cos),
                y: v as f32 * 2. * sin + if h % 2 == 0 { 0. } else { sin },
            }
        }

        // TODO: replace this with manhattan distance
        // approximate straight-line distance in units of side length
        pub fn distance_to(self, HexPoint { h, v }: HexPoint) -> usize {
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
                    let dir2 = dir.next_clockwise();
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

    impl Debug for HexPoint {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            write!(f, "<{}, {}>", self.h, self.v)
        }
    }

    impl PartialOrd for HexPoint {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for HexPoint {
        fn cmp(&self, other: &Self) -> Ordering {
            match self.v.cmp(&other.v) {
                Ordering::Equal => self.h.cmp(&other.h),
                ord => ord,
            }
        }
    }

    impl HexPoint {
        #[must_use]
        pub fn translate(self, dir: Dir, dist: usize) -> Self {
            let dist = dist as isize;
            let mut new_pos = self;
            match dir {
                U => new_pos.v -= dist,
                D => new_pos.v += dist,
                // positive adjustment going up, negative adjustment going down
                UL => {
                    // number of odd columns in range (excluding arrival column)
                    let adjustment = if new_pos.h.is_odd() {
                        (dist + 1) / 2
                    } else {
                        dist / 2
                    };
                    new_pos.h -= dist;
                    new_pos.v -= dist;
                    new_pos.v += adjustment;
                }
                UR => {
                    // number of odd columns in range (excluding arrival column)
                    let adjustment = if new_pos.h.is_odd() {
                        (dist + 1) / 2
                    } else {
                        dist / 2
                    };
                    new_pos.h += dist;
                    new_pos.v -= dist;
                    new_pos.v += adjustment;
                }
                DL => {
                    // number of even columns in range (excluding arrival column)
                    let adjustment = if new_pos.h.is_even() {
                        (dist + 1) / 2
                    } else {
                        dist / 2
                    };
                    new_pos.h -= dist;
                    new_pos.v += dist;
                    new_pos.v -= adjustment;
                }
                DR => {
                    // number of even columns in range (excluding arrival column)
                    let adjustment = if new_pos.h.is_even() {
                        (dist + 1) / 2
                    } else {
                        dist / 2
                    };
                    new_pos.h += dist;
                    new_pos.v += dist;
                    new_pos.v -= adjustment;
                }
            }

            new_pos
        }

        // basically mod width, mod height
        // if the point is n cells out of bounds, it will be n cells from the edge
        // TODO: improve efficiency
        #[must_use]
        pub fn wrap_around(mut self, board_dim: HexDim, axis: Axis) -> Option<Self> {
            use Axis::*;

            if !board_dim.contains(self) {
                // opposite of adjustment direction
                // (direction snake was going)
                #[rustfmt::skip]
                let dir = match axis {
                    UD => if self.v < 0 { U } else { D },
                    ULDR => if self.v < 0 || self.h < 0 { UL } else { DR },
                    URDL => if self.v < 0 || self.h >= board_dim.h { UR } else { DL },
                };

                // check if the point is salvageable, otherwise return None
                {
                    fn problems(point: HexPoint, board_dim: HexDim) -> (bool, bool, bool, bool) {
                        (
                            point.v < 0,
                            point.v >= board_dim.v,
                            point.h < 0,
                            point.h >= board_dim.h,
                        )
                    }

                    let mut x = self;
                    let probs = problems(x, board_dim);
                    while !board_dim.contains(x) {
                        if problems(x, board_dim) != probs {
                            return None;
                        }
                        x = x.translate(-dir, 1);
                    }
                }

                // the board size on that axis at that location
                // e.g. small in corners, constant for U/D, etc.
                let axis_board_size = {
                    let mut x = self;
                    let mut size = 0;
                    while !board_dim.contains(x) {
                        x = x.translate(-dir, 1);
                    }
                    while board_dim.contains(x) {
                        x = x.translate(-dir, 1);
                        size += 1;
                    }
                    size
                };

                while !board_dim.contains(self) {
                    // let d2 = match axis {
                    //     UD => if self.v < 0 { U } else { D },
                    //     ULDR => if self.v < 0 || self.h < 0 { UL } else { DR },
                    //     URDL => if self.v < 0 || self.h >= board_dim.h { UR } else { DL },
                    // };
                    // assert_eq!(dir, d2);

                    self = self.translate(-dir, axis_board_size);
                }
            }

            Some(self)
        }

        // wraps around board edges
        #[must_use]
        pub fn wrapping_translate(self, dir: Dir, dist: usize, board_dim: HexDim) -> Self {
            self.translate(dir, dist)
                .wrap_around(board_dim, dir.axis())
                .unwrap()
        }

        pub fn contains(self, pos: Self) -> bool {
            (0..self.h).contains(&pos.h) && (0..self.v).contains(&pos.v)
        }
    }
}
