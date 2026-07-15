use crate::basic::{Food, HexPoint};
use crate::snake::builder::Builder as SnakeBuilder;

#[macro_use]
pub mod spawn;

#[derive(Debug, Clone)]
pub enum Type {
    Food(Food),
    /// Temporarily speeds up the snake that eats it
    SpeedBoost,
    /// Bursts a shower of extra food apples onto the board
    Frenzy,
    SpawnSnake(Box<SnakeBuilder>),
    SpawnRain,
}

impl Type {
    pub fn is_animated(&self) -> bool {
        match self {
            Type::Food(_) => false,
            Type::SpeedBoost => true,
            Type::Frenzy => true,
            Type::SpawnSnake(_) => true,
            Type::SpawnRain => true,
        }
    }
}

#[derive(Clone)]
pub struct Apple {
    pub pos: HexPoint,
    pub apple_type: Type,
}
