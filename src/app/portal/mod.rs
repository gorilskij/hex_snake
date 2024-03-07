use crate::basic::HexPoint;

// TODO: support negative indices for portals to make them
//       stick to the right or bottom edge

#[derive(Copy, Clone, Debug)]
pub enum Behavior {
    Die,
    TeleportTo(HexPoint),
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

    pub fn cells() -> Vec<Self> {
        let cell1 = Self {
            edges: vec![
                Edge {
                    a: HexPoint { h: 4, v: 6 },
                    b: HexPoint { h: 4, v: 7 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 10, v: 21 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 5, v: 6 },
                    b: HexPoint { h: 4, v: 7 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 9, v: 20 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 5, v: 7 },
                    b: HexPoint { h: 4, v: 7 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 9, v: 19 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 4, v: 8 },
                    b: HexPoint { h: 4, v: 7 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 10, v: 19 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 3, v: 7 },
                    b: HexPoint { h: 4, v: 7 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 11, v: 19 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 3, v: 6 },
                    b: HexPoint { h: 4, v: 7 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 11, v: 20 }),
                    behavior_ba: Behavior::Nothing,
                },
            ],
        };

        let cell2 = Self {
            edges: vec![
                Edge {
                    a: HexPoint { h: 10, v: 19 },
                    b: HexPoint { h: 10, v: 20 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 4, v: 8 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 11, v: 19 },
                    b: HexPoint { h: 10, v: 20 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 3, v: 7 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 11, v: 20 },
                    b: HexPoint { h: 10, v: 20 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 3, v: 6 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 10, v: 21 },
                    b: HexPoint { h: 10, v: 20 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 4, v: 6 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 9, v: 20 },
                    b: HexPoint { h: 10, v: 20 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 5, v: 6 }),
                    behavior_ba: Behavior::Nothing,
                },
                Edge {
                    a: HexPoint { h: 9, v: 19 },
                    b: HexPoint { h: 10, v: 20 },
                    behavior_ab: Behavior::TeleportTo(HexPoint { h: 5, v: 7 }),
                    behavior_ba: Behavior::Nothing,
                },
            ],
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
                    behavior_ba: Behavior::TeleportTo(HexPoint { h: 10, v: 10 }),
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 6, v: 5 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::TeleportTo(HexPoint { h: 10, v: 10 }),
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 7, v: 4 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::TeleportTo(HexPoint { h: 10, v: 10 }),
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 7, v: 3 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::TeleportTo(HexPoint { h: 10, v: 10 }),
                },
                Edge {
                    a: HexPoint { h: 6, v: 4 },
                    b: HexPoint { h: 6, v: 3 },
                    behavior_ab: Behavior::Die,
                    behavior_ba: Behavior::TeleportTo(HexPoint { h: 10, v: 10 }),
                },
            ],
        }]
    }
}
