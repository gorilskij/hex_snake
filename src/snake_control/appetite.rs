use std::collections::HashMap;

use crate::apple::{self, Apple};
use crate::snake_control::pathfinder::Goals;

/// How much an autopilot wants each kind of apple: the apples it goes for and
/// the ones it goes around, as [`Knowledge`] is what it can go through.
///
/// A positive value makes an apple a target. A negative one makes its cell
/// cost that much more to cross, on top of the path's own costs (see
/// [`Weights`]). Zero ignores the apple altogether.
///
/// [`Knowledge`]: crate::snake::eat_mechanics::Knowledge
/// [`Weights`]: crate::snake_control::pathfinder::Weights
#[derive(Copy, Clone, Debug)]
pub struct Appetite {
    pub eat: i32,
    pub grow: i32,
    pub shrink: i32,
    pub spawn_snake: i32,
    pub spawn_rain: i32,
}

impl Default for Appetite {
    /// Every apple is a target.
    fn default() -> Self {
        Self {
            eat: 1,
            grow: 1,
            shrink: 1,
            spawn_snake: 1,
            spawn_rain: 1,
        }
    }
}

impl Appetite {
    pub fn of(&self, apple_type: &apple::Type) -> i32 {
        use apple::Type::*;
        match apple_type {
            Eat(_) => self.eat,
            Grow(_) => self.grow,
            Shrink(_) => self.shrink,
            SpawnSnake(_) => self.spawn_snake,
            SpawnRain => self.spawn_rain,
        }
    }

    /// What a search should go for on this board, and what it should go around.
    pub fn goals(&self, apples: &[Apple]) -> Goals {
        let mut goals = Goals {
            targets: vec![],
            avoid: HashMap::new(),
        };
        for apple in apples {
            match self.of(&apple.apple_type) {
                value if value > 0 => goals.targets.push(apple.pos),
                value if value < 0 => {
                    goals.avoid.insert(apple.pos, value.unsigned_abs());
                }
                _ => {}
            }
        }
        goals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic::HexPoint;

    fn apple(h: isize, apple_type: apple::Type) -> Apple {
        Apple {
            pos: HexPoint { h, v: 0 },
            apple_type,
            time_left: None,
        }
    }

    #[test]
    fn wanted_apples_are_targets_and_unwanted_ones_cost_to_cross() {
        let appetite = Appetite {
            shrink: -15,
            spawn_rain: 0,
            ..Default::default()
        };
        let apples = [
            apple(0, apple::Type::Grow(1.)),
            apple(1, apple::Type::Shrink(1.)),
            apple(2, apple::Type::SpawnRain),
        ];
        let goals = appetite.goals(&apples);
        assert_eq!(goals.targets, [HexPoint { h: 0, v: 0 }]);
        assert_eq!(goals.avoid, HashMap::from([(HexPoint { h: 1, v: 0 }, 15)]));
    }
}
