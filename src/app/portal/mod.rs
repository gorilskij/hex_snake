use crate::basic::{Dir, HexPoint};

// TODO: support negative indices for portals to make them
//       stick to the right or bottom edge

#[derive(Copy, Clone, Debug)]
pub enum Behavior {
    Die,
    TeleportTo(HexPoint, Dir),
    WrapAround,
    PassThrough,
    Nothing,
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

#[derive(Debug)]
pub struct Portal {
    pub edges: Vec<Edge>,
}

impl Portal {
    pub fn check(&self, from: HexPoint, to: HexPoint) -> Option<Behavior> {
        assert_ne!(from, to);
        for edge in &self.edges {
            if edge.a == from && edge.b == to {
                return Some(edge.behavior_ab);
            } else if edge.b == from && edge.a == to {
                return Some(edge.behavior_ba);
            }
        }
        None
    }

    pub fn cells(pos1: HexPoint, pos2: HexPoint) -> Vec<Self> {
        // TODO: make sure the positions don't touch the edges of the board

        let cell1 = Self {
            edges: Dir::iter()
                .map(|dir| Edge {
                    a: pos1 + -dir,
                    b: pos1,
                    behavior_ab: Behavior::TeleportTo(pos2 + dir, dir),
                    behavior_ba: Behavior::Nothing,
                })
                .collect(),
        };

        let cell2 = Self {
            edges: Dir::iter()
                .map(|dir| Edge {
                    a: pos2 + -dir,
                    b: pos2,
                    behavior_ab: Behavior::TeleportTo(pos1 + dir, dir),
                    behavior_ba: Behavior::Nothing,
                })
                .collect(),
        };

        vec![cell1, cell2]
    }

    pub fn cells_inverse(pos1: HexPoint, pos2: HexPoint) -> Vec<Self> {
        // TODO: make sure the positions don't touch the edges of the board

        let cell1 = Self {
            edges: Dir::iter()
                .map(|dir| Edge {
                    a: pos1 + dir,
                    b: pos1,
                    behavior_ab: Behavior::TeleportTo(pos2 + dir, dir),
                    behavior_ba: Behavior::Nothing,
                })
                .collect(),
        };

        let cell2 = Self {
            edges: Dir::iter()
                .map(|dir| Edge {
                    a: pos2 + dir,
                    b: pos2,
                    behavior_ab: Behavior::TeleportTo(pos1 + dir, dir),
                    behavior_ba: Behavior::Nothing,
                })
                .collect(),
        };

        vec![cell1, cell2]
    }

    pub fn test() -> Vec<Self> {
        vec![Self {
            edges: vec![
                Edge {
                    a: HexPoint { h: 4, v: 4 },
                    b: HexPoint { h: 4, v: 5 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
                Edge {
                    a: HexPoint { h: 4, v: 4 },
                    b: HexPoint { h: 5, v: 4 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
                Edge {
                    a: HexPoint { h: 5, v: 3 },
                    b: HexPoint { h: 5, v: 4 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 5, v: 4 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 6, v: 5 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 7, v: 4 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 7, v: 3 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 6, v: 3 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::WrapAround,
                },
            ],
        }]
    }
}
