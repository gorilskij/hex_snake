use crate::basic::HexPoint;

// TODO: support negative indices for portals to make them
//       stick to the right or bottom edge

#[derive(Copy, Clone, Debug)]
pub enum Behavior {
    Die,
    Teleport,
    PassThrough,
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

    pub fn test() -> Self {
        Self {
            edges: vec![Edge {
                a: HexPoint { h: 4, v: 4 },
                b: HexPoint { h: 4, v: 5 },
                behavior_ab: Behavior::Die,
                behavior_ba: Behavior::Teleport,
            }],
        }
    }
}
