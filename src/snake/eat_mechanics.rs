use enum_map_lite::{enum_map, EnumMap};

use crate::snake::{self, Segment, SegmentType};

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

/// The eat behavior against each kind of segment. Build with [`enum_map!`];
/// keying is on the [`SegmentType`] discriminant (fields ignored).
pub type BySegmentType = EnumMap<SegmentType, EatBehavior>;

/// The eat behavior against each kind of snake, per segment type. Build with
/// [`enum_map!`]; keying is on the [`snake::Type`] discriminant.
pub type BySnakeType = EnumMap<snake::Type, BySegmentType>;

#[derive(Copy, Clone, Debug)]
pub struct EatMechanics {
    eat_self: BySegmentType,
    eat_other: BySnakeType,
}

impl EatMechanics {
    pub fn new(eat_self: BySegmentType, eat_other: BySnakeType) -> Self {
        Self { eat_self, eat_other }
    }

    pub fn eat_self(&self, segment_type: SegmentType) -> EatBehavior {
        self.eat_self[segment_type]
    }

    pub fn eat_other(&self, snake_type: snake::Type, segment_type: SegmentType) -> EatBehavior {
        self.eat_other[snake_type][segment_type]
    }

    pub fn always(behavior: EatBehavior) -> Self {
        Self {
            eat_self: enum_map! { _ => behavior },
            eat_other: enum_map! { _ => enum_map! { _ => behavior } },
        }
    }
}

// what an ai algorithm thinks is gonna happen
#[derive(Clone, Debug)]
pub struct Knowledge(EatMechanics);

impl Knowledge {
    // accurate model of a snake's collision behavior
    pub fn accurate(eat_mechanics: &EatMechanics) -> Self {
        Self(*eat_mechanics)
    }

    pub fn always(can_pass_through: bool) -> Self {
        if can_pass_through {
            Self(EatMechanics::always(EatBehavior::PassOver))
        } else {
            Self(EatMechanics::always(EatBehavior::Crash))
        }
    }

    /// Checks whether the snake can safely pass through a given segment
    /// belonging to itself
    pub fn can_pass_through_self(&self, seg: &Segment) -> bool {
        self.0.eat_self(seg.segment_type).is_inert()
    }
}
