use super::dir::{Axis, Dir};
use crate::basic::{CellDim, Point};
use std::{
    cmp::{max, Ordering},
    fmt::{Debug, Error, Formatter},
};
use Dir::*;

// INVARIANT: even columns are half a cell higher than odd columns
#[derive(Eq, PartialEq, Copy, Clone, Div, Add, Hash)]
pub struct HexPoint {
    pub h: isize,
    pub v: isize,
}

pub type HexDim = HexPoint;

impl HexPoint {
    pub fn to_point(self, cell_dim: CellDim) -> Point {
        let Self { h, v } = self;
        let CellDim { side, sin, cos } = cell_dim;
        Point {
            x: h as f32 * (side + cos),
            y: (v as f32 * 2. + (h % 2) as f32) * sin,
        }
    }

    // approximate straight-line distance in units of side length
    // pub fn distance_to(self, other: Self) -> usize {
    //     let dh = (self.h - other.h).abs() as f32;
    //     let dv = (self.v - other.v).abs() as f32;
    //     let CellDim { side, sin, cos } = CellDim::from(1.);
    //     (dh / (side + cos) + dv / (2. * sin)) as usize
    // }

    // None if the two points are not on the same line
    // NOTE: doesn't consider wrapping!
    pub fn dir_to(self, other: Self) -> Option<Dir> {
        if self.h == other.h {
            return Some(if self.v > other.v { U } else { D });
        } else {
            let dh = (self.h - other.h).abs();
            if self.v > other.v || self.v == other.v && self.h % 2 == 1 {
                // going up
                let dv = dh - (dh + self.h % 2) / 2;
                if other.v == self.v - dv {
                    return Some(if self.h > other.h { UL } else { UR });
                }
            } else if self.v < other.v || self.v == other.v && self.h % 2 == 0 {
                // going down
                let dv = dh - (dh + (self.h + 1) % 2) / 2;
                let expected_v = self.v + dv;
                if expected_v == other.v {
                    return Some(if self.h > other.h { DL } else { DR });
                }
            }
        }

        println!("no dir from {:?} to {:?}", self, other);
        None
    }

    // None if the two points are not on the same line or are farther than 1 unit apart
    // This version allows wrapping around the board
    pub fn wrapping_dir_to_1(self, other: Self, board_dim: HexDim) -> Option<Dir> {
        // O(12) goon enough?
        Dir::iter().find(|dir| self.wrapping_translate(*dir, 1, board_dim) == other)
    }

    // O(1)
    pub fn manhattan_distance(self, other: Self) -> usize {
        let dh = (self.h - other.h).abs();
        let max_dv = if self.v > other.v {
            dh - (dh + (self.h % 2).abs()) / 2
        } else {
            dh - (dh + 1 - (self.h % 2).abs()) / 2
        };
        let dv = (self.v - other.v).abs();
        let dv_overflow = max(0, dv - max_dv);
        (dh + dv_overflow) as usize
    }

    // obviously oblivious to teleportation, only works on the plane
    // all cells within a manhattan distance of radius (including self)
    // guarantees no duplicates, not sorted
    // pub fn neighborhood(self, radius: usize) -> Vec<Self> {
    //     let mut neighborhood = vec![self];
    //     for dir in Dir::iter() {
    //         for r in 1..radius {
    //             let branch = self.translate(dir, r);
    //             neighborhood.push(branch);
    //             let dir2 = dir + 1;
    //             for r2 in 1..(radius - r) {
    //                 neighborhood.push(branch.translate(dir2, r2))
    //             }
    //         }
    //     }
    //
    //     // check that no duplicates were introduced
    //     // let len = neighborhood.len();
    //     // neighborhood.sort()
    //     // neighborhood.dedup();
    //     // assert_eq!(neighborhood.len(), len);
    //
    //     neighborhood
    // }
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
        let dh = dist as isize;
        let mut new_pos = self;

        // adjustment:
        //  going from an even column left or right UP means decrementing v
        //  going from an even column left or right DOWN means keeping v as it is
        //   (the next column is already shifted down)
        //  the opposite holds starting at an odd column
        // important: (-1) % 2 == -1
        // that's why .abs()
        match dir {
            U => new_pos.v -= dh,
            D => new_pos.v += dh,
            UL => {
                let adjustment = (dh + (self.h % 2).abs()) / 2;
                let dv = dh - adjustment;
                new_pos.h -= dh;
                new_pos.v -= dv;
            }
            UR => {
                let adjustment = (dh + (self.h % 2).abs()) / 2;
                let dv = dh - adjustment;
                new_pos.h += dh;
                new_pos.v -= dv;
            }
            DL => {
                let adjustment = (dh + 1 - (self.h % 2).abs()) / 2;
                let dv = dh - adjustment;
                new_pos.h -= dh;
                new_pos.v += dv;
            }
            DR => {
                let adjustment = (dh + 1 - (self.h % 2).abs()) / 2;
                let dv = dh - adjustment;
                new_pos.h += dh;
                new_pos.v += dv;
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
                        // println!(
                        //     "problems was {:?} for {:?}, is {:?} for {:?}",
                        //     probs,
                        //     self,
                        //     problems(x, board_dim),
                        //     x
                        // );
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
        let translated = self.translate(dir, dist);
        translated
            .wrap_around(board_dim, dir.axis())
            .unwrap_or_else(|| {
                panic!(
                    "failed to wrap pos: {:?}, translated: {:?} (board_dim: {:?}, dir: {:?})",
                    self, translated, board_dim, dir
                )
            })
    }

    pub fn contains(self, pos: Self) -> bool {
        (0..self.h).contains(&pos.h) && (0..self.v).contains(&pos.v)
    }
}

#[test]
fn test_manhattan_distance() {
    [
        ((0, 0), (0, 0), 0),
        ((0, 0), (0, 1), 1),
        ((0, 0), (1, 0), 1),
        ((0, 0), (0, 10), 10),
        ((0, 0), (0, -10), 10),
        ((0, 10), (0, 0), 10),
        ((0, -10), (0, 0), 10),
        ((1, 1), (2, 2), 1),
        ((1, 1), (3, 3), 3),
    ]
    .iter()
    .for_each(|&((h1, v1), (h2, v2), d)| {
        let p1 = HexPoint { h: h1, v: v1 };
        let p2 = HexPoint { h: h2, v: v2 };
        assert_eq!(p1.manhattan_distance(p2), d);
    });
}
