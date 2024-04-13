use std::cell::{Ref, RefCell};
use std::fmt::{Debug, Formatter};

use num_traits::Zero;

use crate::basic::{Dir, HexDim, HexPoint};

// TODO: support negative indices for portals to make them
//       stick to the right or bottom edge

#[derive(Copy, Clone, Debug)]
pub enum Behavior {
    Die,
    TeleportTo(HexPoint, Dir),
    WrapAround,
    PassThrough,
    Nothing,
    Unreachable,
}

/// An edge is uniquely identified by the two
/// hexagons it touches.
#[derive(Debug)]
pub struct Edge {
    pub a: HexPoint,
    pub b: HexPoint,
    pub behavior_ab: Behavior, // when passing from a to b
    pub behavior_ba: Behavior, // when passing from b to a
}

type GetEdges = Box<dyn Fn(HexDim) -> Vec<Edge>>;

// TODO: make portal template
pub struct Portal {
    // the board dimension that current edges are adapted to
    pub board_dim: HexDim,
    pub get_edges: GetEdges,
    pub edges: Vec<Edge>,
}

impl Debug for Portal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Portal {{\n\tedges: {:?},\n\t...\n}}", self.edges)
    }
}

impl Portal {
    // TODO: template instead of some random dim
    //       this is very hacky
    pub fn new(get_edges: GetEdges) -> Self {
        let board_dim = HexDim { h: 10, v: 10 };
        let edges = get_edges(board_dim);
        Self { board_dim, edges, get_edges }
    }

    pub fn update(&mut self, board_dim: HexDim) {
        if board_dim != self.board_dim {
            self.board_dim = board_dim;
            self.edges = (self.get_edges)(board_dim)
        }
    }

    pub fn check(&self, from: HexPoint, to: HexPoint) -> Option<Behavior> {
        // assert_ne!(from, to);
        for edge in &self.edges {
            if edge.a == from && edge.b == to {
                return Some(edge.behavior_ab);
            } else if edge.b == from && edge.a == to {
                return Some(edge.behavior_ba);
            }
        }
        None
    }

    pub fn cell(pos: HexPoint, dest: HexPoint) -> Self {
        Self::new(Box::new(move |_| {
            Dir::iter()
                .map(|dir| Edge {
                    a: pos - dir,
                    b: pos,
                    behavior_ab: Behavior::TeleportTo(dest + dir, dir),
                    behavior_ba: Behavior::Nothing,
                })
                .collect()
        }))
    }

    pub fn cells_inverse(pos1: HexPoint, pos2: HexPoint) -> Vec<Self> {
        // TODO: make sure the positions don't touch the edges of the board

        vec![
            Portal::new(Box::new(move |_| {
                Dir::iter()
                    .map(move |dir| Edge {
                        a: pos1 + dir,
                        b: pos1,
                        behavior_ab: Behavior::TeleportTo(pos2 + dir, dir),
                        behavior_ba: Behavior::Nothing,
                    })
                    .collect()
            })),
            Portal::new(Box::new(move |_| {
                Dir::iter()
                    .map(move |dir| Edge {
                        a: pos2 + dir,
                        b: pos2,
                        behavior_ab: Behavior::TeleportTo(pos1 + dir, dir),
                        behavior_ba: Behavior::Nothing,
                    })
                    .collect()
            })),
        ]
    }

    pub fn sun_cell() -> Self {
        Portal::new(Box::new(|board_dim| {
            let center = board_dim / 2;

            Dir::iter()
                .flat_map(|dir| {
                    let outer = center.find_border(dir, board_dim);

                    [
                        Edge {
                            a: center + dir,
                            b: center,
                            behavior_ab: Behavior::TeleportTo(outer, -dir),
                            behavior_ba: Behavior::Nothing,
                            // TODO: constructor that checks validity of Unreachable
                            //       and validity of a and b points
                        },
                        Edge {
                            a: outer,
                            b: outer + dir,
                            behavior_ab: Behavior::TeleportTo(center + dir, dir),
                            behavior_ba: Behavior::Unreachable,
                        },
                    ]
                })
                .collect()
        }))
        // vec![
        //     // top
        //     Edge {
        //         a: HexPoint { h: mid_h, v: 7 },
        //         b: HexPoint { h: mid_h, v: 8 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 13, v: 0 }, Dir::D),
        //         behavior_ba: Behavior::Nothing,
        //         // TODO: constructor that checks validity of Unreachable
        //         //       and validity of a and b points
        //     },
        //     Edge {
        //         a: HexPoint { h: 13, v: 0 },
        //         b: HexPoint { h: 13, v: -1 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 13, v: 7 }, Dir::U),
        //         behavior_ba: Behavior::Unreachable,
        //     },
        //     // bottom
        //     Edge {
        //         a: HexPoint { h: 13, v: 9 },
        //         b: HexPoint { h: 13, v: 8 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 13, v: 16 }, Dir::U),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     Edge {
        //         a: HexPoint { h: 13, v: 16 },
        //         b: HexPoint { h: 13, v: 17 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 13, v: 9 }, Dir::D),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     // top-right
        //     Edge {
        //         a: HexPoint { h: 14, v: 8 },
        //         b: HexPoint { h: 13, v: 8 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 25, v: 2 }, Dir::Dl),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     Edge {
        //         a: HexPoint { h: 25, v: 2 },
        //         b: HexPoint { h: 26, v: 2 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 14, v: 8 }, Dir::Ur),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     // bottom-right
        //     Edge {
        //         a: HexPoint { h: 14, v: 9 },
        //         b: HexPoint { h: 13, v: 8 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 25, v: 14 }, Dir::Ul),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     Edge {
        //         a: HexPoint { h: 25, v: 14 },
        //         b: HexPoint { h: 26, v: 15 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 14, v: 9 }, Dir::Dr),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     // top-left
        //     Edge {
        //         a: HexPoint { h: 12, v: 8 },
        //         b: HexPoint { h: 13, v: 8 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 0, v: 2 }, Dir::Dr),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     Edge {
        //         a: HexPoint { h: 0, v: 2 },
        //         b: HexPoint { h: -1, v: 1 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 12, v: 8 }, Dir::Ul),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     // bottom-left
        //     Edge {
        //         a: HexPoint { h: 12, v: 9 },
        //         b: HexPoint { h: 13, v: 8 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 0, v: 14 }, Dir::Ur),
        //         behavior_ba: Behavior::Nothing,
        //     },
        //     Edge {
        //         a: HexPoint { h: 0, v: 14 },
        //         b: HexPoint { h: -1, v: 14 },
        //         behavior_ab: Behavior::TeleportTo(HexPoint { h: 12, v: 9 }, Dir::Dl),
        //         behavior_ba: Behavior::Nothing,
        //     },
        // ]
    }

    // pub fn test() -> Vec<Self> {
    //     vec![Self {
    //         edges: Box::new(|board_dim| vec![
    //             Edge {
    //                 a: HexPoint { h: 4, v: 4 },
    //                 b: HexPoint { h: 4, v: 5 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //             Edge {
    //                 a: HexPoint { h: 4, v: 4 },
    //                 b: HexPoint { h: 5, v: 4 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //             Edge {
    //                 a: HexPoint { h: 5, v: 3 },
    //                 b: HexPoint { h: 5, v: 4 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //             Edge {
    //                 a: HexPoint { h: 6, v: 4 },
    //                 b: HexPoint { h: 5, v: 4 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //             Edge {
    //                 a: HexPoint { h: 6, v: 4 },
    //                 b: HexPoint { h: 6, v: 5 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //             Edge {
    //                 a: HexPoint { h: 6, v: 4 },
    //                 b: HexPoint { h: 7, v: 4 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //             Edge {
    //                 a: HexPoint { h: 6, v: 4 },
    //                 b: HexPoint { h: 7, v: 3 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //             Edge {
    //                 a: HexPoint { h: 6, v: 4 },
    //                 b: HexPoint { h: 6, v: 3 },
    //                 behavior_ab: Behavior::Die,
    //                 behavior_ba: Behavior::WrapAround,
    //             },
    //         ]),
    //     }]
    // }
}
