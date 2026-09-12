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
    /// See [`EatMechanics::is_marked`].
    mark_passable: bool,
}

impl EatMechanics {
    pub fn new(eat_self: BySegmentType, eat_other: BySnakeType) -> Self {
        Self {
            eat_self,
            eat_other,
            mark_passable: false,
        }
    }

    /// Mark the snake's own segments that it can pass through, to tell them
    /// apart from look-alikes it can't (e.g. other snakes' eaten segments).
    pub fn mark_passable(mut self) -> Self {
        self.mark_passable = true;
        self
    }

    pub fn eat_self(&self, segment_type: SegmentType) -> EatBehavior {
        self.eat_self[segment_type]
    }

    pub fn eat_other(&self, snake_type: snake::Type, segment_type: SegmentType) -> EatBehavior {
        self.eat_other[snake_type][segment_type]
    }

    /// Whether the snake's own segment of this type is drawn with a mark. A
    /// marked segment can always be passed through; not every one that can be
    /// passed through is marked.
    pub fn is_marked(&self, segment_type: SegmentType) -> bool {
        self.mark_passable && self.eat_self(segment_type).is_inert()
    }

    pub fn always(behavior: EatBehavior) -> Self {
        Self {
            eat_self: enum_map! { _ => behavior },
            eat_other: enum_map! { _ => enum_map! { _ => behavior } },
            mark_passable: false,
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
