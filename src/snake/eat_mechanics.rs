use crate::snake::{self, Segment, SegmentRawType};
use crate::support::map_with_default::HashMapWithDefault;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EatBehavior {
    Cut,       // cut the other snake's tail off
    Crash,     // stop the game
    Die,       // disappear
    PassUnder, // pass under the other snake
    PassOver,  // pass over the other snake
}

impl EatBehavior {
    pub fn is_inert(self) -> bool {
        matches!(self, EatBehavior::PassUnder | EatBehavior::PassOver)
    }
}

// TODO: if this is too big, make it shared between snakes as an Rc
#[derive(Clone, Debug)]
pub struct EatMechanics {
    pub eat_self: HashMapWithDefault<SegmentRawType, EatBehavior>,
    pub eat_other: HashMapWithDefault<snake::Type, HashMapWithDefault<SegmentRawType, EatBehavior>>,
}

impl EatMechanics {
    pub fn always(behavior: EatBehavior) -> Self {
        Self {
            eat_self: HashMapWithDefault::new(behavior),
            eat_other: HashMapWithDefault::new(HashMapWithDefault::new(behavior)),
        }
    }
}

// information an ai algorithm has about which types of segments it can pass through
#[derive(Clone, Debug)]
pub struct PassthroughKnowledge {
    through_self: HashMapWithDefault<SegmentRawType, bool>,
    through_other: HashMapWithDefault<snake::Type, HashMapWithDefault<SegmentRawType, bool>>,
}

pub struct Checker<'a>(&'a HashMapWithDefault<SegmentRawType, bool>);

impl PassthroughKnowledge {
    // accurate model of a snake's collision behavior
    pub fn accurate(eat_mechanics: &EatMechanics) -> Self {
        Self {
            through_self: eat_mechanics.eat_self.map_values(|v| v.is_inert()),
            through_other: eat_mechanics
                .eat_other
                .map_values(|map| map.map_values(|v| v.is_inert())),
        }
    }

    pub fn always(can_pass_through: bool) -> Self {
        Self {
            through_self: hash_map_with_default! { default => can_pass_through },
            through_other: hash_map_with_default! {
                default => hash_map_with_default! {
                    default => can_pass_through,
                },
            },
        }
    }

    /// Checks whether the snake can safely pass through a given segment
    /// belonging to itself
    pub fn can_pass_through_self(&self, seg: &Segment) -> bool {
        self.through_self[&seg.segment_type.raw_type()]
    }

    pub fn checker<'a>(&'a self, snake_type: &'a snake::Type) -> Checker<'a> {
        Checker(&self.through_other[snake_type])
    }
}

impl Checker<'_> {
    pub fn can_pass_through_other(&self, seg: &Segment) -> bool {
        self.0[&seg.segment_type.raw_type()]
    }
}
