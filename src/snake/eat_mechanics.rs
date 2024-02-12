use crate::snake::{self, Segment, SegmentType};
use std::mem::Discriminant;

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

// NOTE: for repeated keys, the last value will be used
#[macro_export]
macro_rules! by_segment_type {
    (@ $this:ident; _ => $val:expr $(, $($rest:tt)* )?) => {{
        $this.set(None, $val);
        by_segment_type!(@ $this; $( $($rest)* )?);
    }};
    (@ $this:ident; $variant:path => $val:expr $(, $($rest:tt)* )?) => {{
        $this.set(Some($variant), $val);
        by_segment_type!(@ $this; $( $($rest)* )?);
    }};
    (@ $this:ident;) => {};
    (@ $($wrong:tt)*) => { compile_error!(stringify!($($wrong)*)) };
    ($($rest:tt)*) => {{
        let mut this = $crate::snake::eat_mechanics::BySegmentType::default();
        by_segment_type!(@ this; $($rest)*);
        this
    }};
}

#[macro_export]
macro_rules! by_snake_type {
    (@ $this:ident; _ => $val:expr $(, $($rest:tt)* )?) => {{
        *$this.get_mut(None) = $val;
        by_snake_type!(@ $this; $( $($rest)* )?);
    }};
    (@ $this:ident; $variant:path => $val:expr $(, $($rest:tt)* )?) => {{
        *$this.get_mut(Some($variant)) = $val;
        by_snake_type!(@ $this; $( $($rest)* )?);
    }};
    (@ $this:ident;) => {};
    (@ $($wrong:tt)*) => { compile_error!(stringify!($($wrong)*)) };
    ($($rest:tt)*) => {{
        let mut this = $crate::snake::eat_mechanics::BySnakeType::default();
        by_snake_type!(@ this; $($rest)*);
        this
    }};
}

// INVARIANT: at least one field is Some
#[derive(Default, Debug, Copy, Clone)]
pub struct BySegmentType {
    normal: Option<EatBehavior>,
    eaten: Option<EatBehavior>,
    crashed: Option<EatBehavior>,
    black_hole: Option<EatBehavior>,
    default: Option<EatBehavior>,
}

impl BySegmentType {
    pub fn get(&self, segment_type: Discriminant<SegmentType>) -> EatBehavior {
        let field = match segment_type {
            d if d == SegmentType::DISCR_NORMAL => self.normal,
            d if d == SegmentType::DISCR_EATEN => self.eaten,
            d if d == SegmentType::DISCR_CRASHED => self.crashed,
            d if d == SegmentType::DISCR_BLACK_HOLE => self.black_hole,
            _ => unreachable!(),
        };
        field
            .or(self.default)
            .expect("BROKE INVARIANT: BySegmentType must have at least one field that is Some")
    }

    // pass None to set the default
    pub fn set(&mut self, segment_type: Option<Discriminant<SegmentType>>, behavior: EatBehavior) {
        let value = match segment_type {
            Some(d) if d == SegmentType::DISCR_NORMAL => &mut self.normal,
            Some(d) if d == SegmentType::DISCR_EATEN => &mut self.eaten,
            Some(d) if d == SegmentType::DISCR_CRASHED => &mut self.crashed,
            Some(d) if d == SegmentType::DISCR_BLACK_HOLE => &mut self.black_hole,
            None => &mut self.default,
            _ => unreachable!(),
        };
        *value = Some(behavior);
    }
}

// INVARIANT: at least one field is Some
#[derive(Default, Debug, Copy, Clone)]
pub struct BySnakeType {
    player: Option<BySegmentType>,
    simulated: Option<BySegmentType>,
    competitor: Option<BySegmentType>,
    killer: Option<BySegmentType>,
    rain: Option<BySegmentType>,
    default: Option<BySegmentType>,
}

impl BySnakeType {
    pub fn get(&self, snake_type: snake::Type) -> BySegmentType {
        use snake::Type::*;
        let by_segment_type = match snake_type {
            Player => self.player,
            Simulated => self.simulated,
            Competitor { .. } => self.competitor,
            Killer { .. } => self.killer,
            Rain => self.rain,
        };
        by_segment_type
            .or(self.default)
            .expect("BROKE INVARIANT: BySnakeType must have at least one field that is Some")
    }

    // insert if missing
    pub fn get_mut(&mut self, snake_type: Option<snake::Type>) -> &mut BySegmentType {
        use snake::Type::*;
        let by_segment_type = match snake_type {
            Some(Player) => &mut self.player,
            Some(Simulated) => &mut self.simulated,
            Some(Competitor { .. }) => &mut self.competitor,
            Some(Killer { .. }) => &mut self.killer,
            Some(Rain) => &mut self.rain,
            None => &mut self.default,
        };
        by_segment_type.get_or_insert_with(Default::default)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EatMechanics {
    eat_self: BySegmentType,
    eat_other: BySnakeType,
}

impl EatMechanics {
    pub fn new(eat_self: BySegmentType, eat_other: BySnakeType) -> Self {
        // assert that eat_self and eat_other are not empty
        eat_self.get(SegmentType::DISCR_NORMAL);
        eat_other.get(snake::Type::Player);
        Self { eat_self, eat_other }
    }

    pub fn eat_self(&self, segment_type: Discriminant<SegmentType>) -> EatBehavior {
        self.eat_self.get(segment_type)
    }

    pub fn eat_other(
        &self,
        snake_type: snake::Type,
        segment_type: Discriminant<SegmentType>,
    ) -> EatBehavior {
        self.eat_other.get(snake_type).get(segment_type)
    }
}

// #[test]
// fn test() {
//     println!("{}", std::mem::size_of::<EatMechanics>());
//     println!("{}", std::mem::size_of::<BySegmentType>());
//     println!("{}", std::mem::size_of::<BySnakeType>());
// }

impl EatMechanics {
    pub fn always(behavior: EatBehavior) -> Self {
        Self {
            eat_self: by_segment_type! {
                _ => behavior,
            },
            eat_other: by_snake_type! {
                _ => by_segment_type! {
                    _ => behavior,
                },
            },
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
        self.0.eat_self(seg.segment_type.discriminant()).is_inert()
    }
}
