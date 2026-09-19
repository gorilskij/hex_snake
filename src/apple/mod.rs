use std::time::Duration;

use crate::basic::HexPoint;
use crate::snake::builder::Builder as SnakeBuilder;

#[macro_use]
pub mod spawn;

#[derive(Debug, Clone)]
pub enum Type {
    /// Eaten and digested: grows the snake as the eaten segment reaches the
    /// tail (classic mode)
    Eat(f32),
    /// Grows the snake right away (hunger mode)
    Grow(f32),
    Shrink(f32),
    SpawnSnake(Box<SnakeBuilder>),
    SpawnRain,
}

impl Type {
    pub fn is_animated(&self) -> bool {
        // TODO: a) check if this is still relevant, b) make this dependent on palette
        true
        // match self {
        //     Type::Grow(_) => false,
        //     Type::SpawnSnake(_) => true,
        //     Type::SpawnRain => true,
        // }
    }
}

#[derive(Clone)]
pub struct Apple {
    pub pos: HexPoint,
    pub apple_type: Type,
    /// Game time left before the apple disappears on its own, for apples that
    /// do. Those don't count towards the spawn policy's apple count.
    pub time_left: Option<Duration>,
}
